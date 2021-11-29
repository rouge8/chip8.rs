struct CPU {
    registers: [u8; 16],
    position_in_memory: usize,
    memory: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: usize,
}

impl CPU {
    fn read_opcode(&self) -> u16 {
        let p = self.position_in_memory;
        let op_byte1 = self.memory[p] as u16;
        let op_byte2 = self.memory[p + 1] as u16;

        op_byte1 << 8 | op_byte2
    }

    fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.position_in_memory += 2;

            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;

            let kk = (opcode & 0x00FF) as u8;
            let op_minor = (opcode & 0x000F) as u8;
            let addr = opcode & 0x0FFF; // Also known as `nnn`

            match opcode {
                0x0000 => return,
                0x00E0 => { /* CLEAR SCREEN */ }
                0x00EE => self.ret(),
                0x1000..=0x1FFF => self.jmp(addr),
                0x2000..=0x2FFF => self.call(addr),
                0x3000..=0x3FFF => self.se(x, kk),
                0x4000..=0x4FFF => self.sne(x, kk),
                0x5000..=0x5FFF => self.se(x, y), // Skip next instruction if `Vx = Vy`.
                0x8000..=0x8FFF => match op_minor {
                    4 => self.add_xy(x, y),
                    _ => todo!("opcode: {:04x}", opcode),
                },
                _ => todo!("opcode: {:04x}", opcode),
            }
        }
    }

    /// Return from a subroutine.
    ///
    /// The interpreter sets the program counter to the address at the top of the stack, then
    /// subtracts 1 from the stack pointer.
    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        let addr = self.stack[self.stack_pointer];
        self.position_in_memory = addr as usize;
    }

    /// Jump to location `nnn`.
    ///
    /// The interpreter sets the program counter to `nnn`.
    fn jmp(&mut self, addr: u16) {
        self.position_in_memory = addr as usize;
    }

    /// Call subroutine at `nnn`.
    ///
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the
    /// stack. The PC is then set to `nnn`.
    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow!");
        }

        stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;
        self.position_in_memory = addr as usize;
    }

    /// Skip next instruction if `Vx = kk`.
    ///
    /// The interpreter compares register `Vx` to `kk`, and if they are equal, increments the
    /// program counter by 2.
    fn se(&mut self, vx: u8, kk: u8) {
        if vx == kk {
            self.position_in_memory += 2;
        }
    }

    /// Skip next instruction if `Vx != kk`.
    ///
    /// The interpreter compares register `Vx` to `kk`, and if they are not equal, increments the
    /// program counter by 2.
    fn sne(&mut self, vx: u8, kk: u8) {
        if vx != kk {
            self.position_in_memory += 2;
        }
    }

    /// Set `Vx = Vx + Vy`, set `VF = carry`.
    ///
    /// The values of `Vx` and `Vy` are added together. If the result is greater than 8 bits
    /// (i.e., > 255,) `VF` is set to 1, otherwise 0. Only the lowest 8 bits of the result are
    /// kept, and stored in `Vx`.
    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }
}

fn main() {
    let mut cpu = CPU {
        registers: [0; 16],
        memory: [0; 4096],
        position_in_memory: 0,
        stack: [0; 16],
        stack_pointer: 0,
    };

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    let mem = &mut cpu.memory;

    // CALL the function at 0x100
    mem[0x000] = 0x21;
    mem[0x001] = 0x00;

    // CALL the function at 0x100
    mem[0x002] = 0x21;
    mem[0x003] = 0x00;

    // HALT
    mem[0x004] = 0x00;
    mem[0x005] = 0x00;

    // ADD register 1's value to register 0
    mem[0x100] = 0x80;
    mem[0x101] = 0x14;

    // ADD register 1's value to register 0
    mem[0x102] = 0x80;
    mem[0x103] = 0x14;

    // RETURN
    mem[0x104] = 0x00;
    mem[0x105] = 0xEE;

    cpu.run();

    assert_eq!(cpu.registers[0], 45);

    println!("5 + (10 * 2) + (10 * 2) = {}", cpu.registers[0]);
}
