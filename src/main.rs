use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

mod ops;
use ops::Op;

struct Registers {
    r: Vec<u16>,
    pc: u16,
    n: bool,
    z: bool,
    p: bool,
}

impl Registers {
    fn new() -> Registers {
        return Registers {
            r: vec![0; 8],
            pc: 0,
            n: false,
            z: false,
            p: false,
        };
    }

    fn get(&self, index: u16) -> u16 {
        return self.r[index as usize];
    }

    fn set(&mut self, index: u16, value: u16) {
        self.r[index as usize] = value;
        if (value as i16) < 0 {
            self.n = true;
            self.z = false;
            self.p = false;
        } else if value == 0 {
            self.n = false;
            self.z = true;
            self.p = false;
        } else {
            self.n = false;
            self.z = false;
            self.p = true;
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (i, &v) in self.r.iter().enumerate() {
            fmt.write_fmt(format_args!("r{}=0x{:x} ", i, v))?;
        }
        fmt.write_fmt(format_args!("pc=0x{:x} ", self.pc))?;
        fmt.write_fmt(format_args!("n={} ", self.n))?;
        fmt.write_fmt(format_args!("z={} ", self.z))?;
        fmt.write_fmt(format_args!("p={}", self.p))?;
        Ok(())
    }
}

const KB_STATUS: u16 = 0xfe00;
const KB_DATA: u16 = 0xfe02;

#[derive(Debug)]
struct Memory {
    memory: Vec<u16>,
}

impl Memory {
    fn new() -> Memory {
        return Memory {
            memory: vec![0; 2usize.pow(16)],
        };
    }

    fn load(&mut self, address: u16) -> u16 {
        if address == KB_STATUS {
            self.memory[KB_STATUS as usize] = 1 << 15;
            self.memory[KB_DATA as usize] = getchar();
        } else {
            self.memory[KB_STATUS as usize] = 0;
        }
        return self.memory[address as usize];
    }

    fn store(&mut self, address: u16, value: u16) {
        self.memory[address as usize] = value;
    }

    fn copy(&mut self, base: u16, block: &[u16]) {
        for (offset, word) in block.iter().enumerate() {
            self.store(base + offset as u16, *word);
        }
    }
}

fn load_executable(filename: &str) -> io::Result<Vec<u16>> {
    let mut file = File::open(filename)?;
    let mut bytes: Vec<u8> = vec![];
    file.read_to_end(&mut bytes)?;
    let mut executable: Vec<u16> = vec![];
    for highlow in bytes.chunks(2) {
        match highlow {
            &[high, low] => executable.push(((high as u16) << 8) | (low as u16)),
            _ => return Err(io::Error::new(io::ErrorKind::Other, "Bad file")),
        }
    }
    return Ok(executable);
}

fn getchar() -> u16 {
    // TODO make this non-blocking
    print!("Waiting on input: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    return input.chars().nth(0).unwrap() as u16;
}

fn main() {
    let debug = false;
    let filename = env::args().nth(1).expect("Supply executable filename");
    let executable = load_executable(&filename).expect("Failed to read executable");
    let (base, code) = match executable.split_first() {
        Some((base, code)) => (base, code),
        _ => panic!("Short file"),
    };

    let mut regs = Registers::new();
    let mut mem = Memory::new();
    mem.copy(*base, &code);
    regs.pc = *base;
    println!(
        "Loaded {} ({} instructions) onto base 0x{:x}",
        filename,
        code.len(),
        base
    );
    loop {
        let instr = mem.load(regs.pc);
        regs.pc += 1;
        let decoded = ops::decode(instr).unwrap();
        if debug {
            println!("{:?} 0x{:x} (0b{:b})", decoded, instr, instr);
        }
        match decoded {
            Op::Nop => (),
            Op::Not { dst, src } => regs.set(dst, !regs.get(src)),
            Op::AddReg { dst, src1, src2 } => {
                regs.set(dst, regs.get(src1).wrapping_add(regs.get(src2)));
            }
            Op::AddImm { dst, src, imm } => {
                regs.set(dst, regs.get(src).wrapping_add(imm as u16));
            }
            Op::AndReg { dst, src1, src2 } => {
                regs.set(dst, regs.get(src1) & regs.get(src2));
            }
            Op::AndImm { dst, src, imm } => {
                regs.set(dst, regs.get(src) & imm as u16);
            }
            Op::Load { dst, offset } => {
                regs.set(dst, mem.load(regs.pc.wrapping_add(offset as u16)));
            }
            Op::LoadInd { dst, offset } => {
                let addr = mem.load(regs.pc.wrapping_add(offset as u16));
                regs.set(dst, mem.load(addr));
            }
            Op::LoadReg { dst, base, offset } => {
                let addr = regs.get(base).wrapping_add(offset as u16);
                regs.set(dst, mem.load(addr));
            }
            Op::LoadEffAddr { dst, offset } => {
                let addr = regs.pc.wrapping_add(offset as u16);
                regs.set(dst, addr);
            }
            Op::Store { src, offset } => {
                mem.store(regs.pc.wrapping_add(offset as u16), regs.get(src));
            }
            Op::StoreInd { src, offset } => {
                let addr = mem.load(regs.pc.wrapping_add(offset as u16));
                mem.store(addr, regs.get(src));
            }
            Op::StoreReg { src, base, offset } => {
                let addr = regs.get(base).wrapping_add(offset as u16);
                mem.store(addr, regs.get(src));
            }
            Op::Call { offset } => {
                regs.set(7, regs.pc);
                regs.pc = regs.pc.wrapping_add(offset as u16);
            }
            Op::CallReg { src } => {
                regs.pc = regs.get(src);
            }
            Op::Branch { n, z, p, offset } => {
                if n && regs.n || z && regs.z || p && regs.p {
                    regs.pc = regs.pc.wrapping_add(offset as u16);
                }
            }
            Op::Jump { base } => {
                regs.pc = regs.get(base);
            }
            Op::Trap { vector } => match vector {
                0x20 => {
                    // getc
                    regs.set(0, getchar());
                }
                0x21 => {
                    // putc
                    print!("{}", regs.get(0) as u8 as char);
                    std::io::stdout().flush().unwrap();
                }
                0x22 => {
                    // puts
                    let mut i = regs.get(0);
                    loop {
                        let c = mem.load(i);
                        match c {
                            0 => break,
                            c => print!("{}", ((c & 0xff) as u8) as char),
                        }
                        i += 1;
                    }
                    std::io::stdout().flush().unwrap();
                }
                0x25 => {
                    println!("Halt");
                    break;
                }
                _ => panic!("Unknown trap vector {:x}", vector),
            },
        }
        if debug {
            println!("{}", regs);
            let mut _s = String::new();
            std::io::stdin().read_line(&mut _s).unwrap();
        }
    }
}
