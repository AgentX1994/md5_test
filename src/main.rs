const SHIFT_BY_ROUND: [u32;64] = [
    7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,
    5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,
    4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,
    6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21];

// Calculate constant K
// should be a const fn, but those are not yet stable
fn calc_k() -> [u32;64] {
    let mut temp = [0u32; 64];
    let mut i = 0;
    while i < 64 {
        temp[i] = (2f64.powi(32) * ((i+1) as f64).sin().abs()) as u32;
        i += 1;
    }
    return temp;
}

// functions used to mix B, C, and D
#[allow(non_snake_case)]
fn F(B: u32, C: u32, D: u32) -> u32 {
    return (B & C) | (!B & D);
}

#[allow(non_snake_case)]
fn G(B: u32, C: u32, D: u32) -> u32 {
    return (B & D) | (C & !D);
}

#[allow(non_snake_case)]
fn H(B: u32, C: u32, D: u32) -> u32 {
    return B ^ C ^ D;
}

#[allow(non_snake_case)]
fn I(B: u32, C: u32, D: u32) -> u32 {
    return C ^ (B | !D);
}

fn u64_to_u8_array(x: u64) -> [u8;8] {
    let b1 = ((x >> 56) & 0xff) as u8;
    let b2 = ((x >> 48) & 0xff) as u8;
    let b3 = ((x >> 40) & 0xff) as u8;
    let b4 = ((x >> 32) & 0xff) as u8;
    let b5 = ((x >> 24) & 0xff) as u8;
    let b6 = ((x >> 16) & 0xff) as u8;
    let b7 = ((x >>  8) & 0xff) as u8;
    let b8 = ((x      ) & 0xff) as u8;
    [b1, b2, b3, b4, b5, b6, b7, b8]
}

fn u8_block_to_u32_block(block: &[u8]) -> [u32; 16] {
    assert_eq!(block.len(), 64);
    let mut ret = [0u32; 16];

    for i in 0..16 {
        ret[i] = ((block[  i*4] as u32)      )
               + ((block[i*4+1] as u32) <<  8)
               + ((block[i*4+2] as u32) << 16)
               + ((block[i*4+3] as u32) << 24);
    }
    
    ret
}

// starting values for state variables
const A0: u32 = 0x67452301;
const B0: u32 = 0xEFCDAB89;
const C0: u32 = 0x98BADCFE;
const D0: u32 = 0x10325476;

fn md5(message: &str) -> u128 {
    #[allow(non_snake_case)]
    let K = calc_k();

    #[cfg(debug_assertions)]
    println!("\tCalculating md5 of {:?}", message);
    #[cfg(debug_assertions)]
    println!("\tlen is {}", message.len());
    // copy message into new buffer
    let mut m = message.to_string().into_bytes();
    let message_length_bytes: u32 = (m.len()*std::mem::size_of::<u8>()) as u32; 
    let message_length_bits       = (message_length_bytes as u64)*8; 
    let padding_length_bits       = ((448i64 - ((message_length_bits+1) % 512) as i64) + 512 % 512) as u64 + 1;
    let padding_length_bytes      = padding_length_bits/8;

    // pad message
    let mut first = true;
    for _ in 0..(padding_length_bytes) {
        if first {
            m.push(0x80);
            first = false;
        } else {
            m.push(0x00);
        }
    }

    #[cfg(debug_assertions)]
    println!("\tPadding length is {} bytes", padding_length_bytes);

    // push original message length in bits
    let bytes = u64_to_u8_array(message_length_bits);
    m.push(bytes[7]);
    m.push(bytes[6]);
    m.push(bytes[5]);
    m.push(bytes[4]);
    m.push(bytes[3]);
    m.push(bytes[2]);
    m.push(bytes[1]);
    m.push(bytes[0]);

    #[cfg(debug_assertions)]
    println!("\tTotal number of blocks: {}", m.len()/64);

    assert_eq!(m.len()*std::mem::size_of::<u8>()*8 % 512, 0);

    // process the message
    
    let mut a0 = A0;
    let mut b0 = B0;
    let mut c0 = C0;
    let mut d0 = D0;
    #[cfg(debug_assertions)]
    println!("m len in bits: {:X}", message_length_bits);
    #[cfg(debug_assertions)]
    println!("m = {:X?}", m);
    for block_num in 0..(m.len()*std::mem::size_of::<u8>()/64) {
        let block = &m[block_num..block_num+64];
        #[cfg(debug_assertions)]
        println!("\tblock        = {:X?}", block);
        let block_as_u32 = u8_block_to_u32_block(block);
        #[cfg(debug_assertions)]
        println!("\tblock_as_u32 = {:08X?}", block_as_u32);

        #[allow(non_snake_case)]
        let mut A = a0;
        #[allow(non_snake_case)]
        let mut B = b0;
        #[allow(non_snake_case)]
        let mut C = c0;
        #[allow(non_snake_case)]
        let mut D = d0;

        #[cfg(debug_assertions)]
        println!("\tBefore all rounds, A = {:08X} B = {:08X} C = {:08X} D = {:08X}", A, B, C, D);
        for i in 0..64 {
            let mut mixer: u32;
            let block_index: u32;

            if i < 16 {
                mixer = F(B,C,D);
                block_index = i;
            } else if 16 <= i && i < 32 {
                mixer = G(B,C,D);
                block_index = (5*i + 1) % 16;
            } else if 32 <= i && i < 48 {
                mixer = H(B,C,D);
                block_index = (3*i + 5) % 16;
            } else if 48 <= i && i < 64 {
                mixer = I(B,C,D);
                block_index = (7*i) % 16;
            } else {
                unreachable!()
            }
            mixer = mixer
                .wrapping_add(A)
                .wrapping_add(K[i as usize])
                .wrapping_add(block_as_u32[block_index as usize]);
            A = D;
            D = C;
            C = B;
            B = B.wrapping_add(mixer.rotate_left(SHIFT_BY_ROUND[i as usize]));
            #[cfg(debug_assertions)]
            println!("\ti = {}, A = {:08X} B = {:08X} C = {:08X} D = {:08X}, K[i] = {:08X}, shift = {}", i, A, B, C, D, K[i as usize], SHIFT_BY_ROUND[i as usize]);
        }
        a0 = a0.wrapping_add(A);
        b0 = b0.wrapping_add(B);
        c0 = c0.wrapping_add(C);
        d0 = d0.wrapping_add(D);
        #[cfg(debug_assertions)]
        println!("\tAfter adding original values: a0 = {:08X} b0 = {:08X} c0 = {:08X} d0 = {:08X}", a0, b0, c0, d0);
    }
    
    let hash: u128 = ((a0.swap_bytes() as u128) << 96) + ((b0.swap_bytes() as u128) << 64) + ((c0.swap_bytes() as u128) << 32) + (d0.swap_bytes() as u128);
    return hash;
}

fn md5_test(message: &str) {
    println!("{:?}", message);
    println!("{:032X?}", md5(&message));
}

fn main() {
    md5_test("");
    md5_test("a");
    md5_test("abc");
    md5_test("The quick brown fox jumps over the lazy dog");
    md5_test("The quick brown fox jumps over the lazy dog.");
}
