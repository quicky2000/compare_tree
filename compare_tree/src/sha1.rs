use std::fmt;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct Sha1Key {
    words: [u32; 5]
}

impl fmt::Display for Sha1Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.words.iter().map(|x| write!(f, "{:08X}", x)).rev().collect()
    }
}

impl Sha1Key {
    fn from_array(words: [u32; 5]) -> Sha1Key {
        Sha1Key {words: words}
    }
}

fn f( x: u32
    , y: u32
    , z: u32
    , i: u32
    ) -> u32 {
    match i / 20 {
        0 => ch(x, y, z),
        1| 3 => parity(x, y, z),
        2 =>  maj(x, y, z),
        _ => panic!("Should never occur as is in range [0:79]")
    }
}

//------------------------------------------------------------------------------
fn ch( x: u32
     , y: u32
     , z: u32
     ) -> u32 {
    (x & y ) ^ ( (!x) & z )
}

//------------------------------------------------------------------------------
fn parity( x: u32
         , y: u32
         , z: u32
         ) -> u32 {
    x ^ y ^ z
}

//------------------------------------------------------------------------------
fn maj( x: u32
      , y: u32
      , z: u32
      ) -> u32 {
    (x & y ) ^ ( x & z ) ^ ( y & z )
}

//------------------------------------------------------------------------------
fn display_block(block: &[u32; 16]) {
    println!("------------------------");
    // Display block
    for (index, word) in block.iter().enumerate() {
        println!("Word[{}] = 0x{:X}", index, word) ;
    }
}

//------------------------------------------------------------------------------
pub fn compute_sha1(data: Vec<u8>) -> Sha1Key {
    let mut key: [u32; 5] = [0x67452301
                            ,0xefcdab89
                            ,0x98badcfe
                            ,0x10325476
                            ,0xc3d2e1f0
                            ];

    let size_bit: u64 = data.len() as u64 * u64::from(u8::BITS);

    if cfg!(test) { println!("Size in bit = {} => 0x{:X}", size_bit, size_bit); }

    // Computing length complement
    let complement_size: u64 = (448u64.wrapping_sub(size_bit + 1)) % 512;

    if cfg!(test) { println!("Number of padding zeros = {}", complement_size); }

    // Computing number of blocks
    let nb_blocks: u64 = (size_bit + 1 + complement_size + 64) / 512;

    if cfg!(test) { println!("Nb blocks = {}", nb_blocks); }

    for block_index in 0..nb_blocks {

        // Working block
        let mut working_block: [u32; 16] = [0; 16];

        // Init block with datas
        //--------------------------
        if cfg!(test) { println!("Prepare block[{}]", block_index); }
        // Check if this is a complete block
        if cfg!(test) { println!("check if this is a complete block"); }
        if (block_index + 1) * 512 <= size_bit {
            if cfg!(test) { println!("Copy complete block"); }
            for word_index in 0..16 {
                working_block[word_index] = u32::from_be_bytes(data[word_index * 4..word_index* 4 + 4].try_into().unwrap());
            }
            if cfg!(test) { display_block(&working_block); }
        }
        else {
            let remaining_size_bits = (size_bit % 512) as u32;
            if cfg!(test) { println!("size of incomplete block in bits = {}", remaining_size_bits); }
            let l_rest_size_word = 1 + remaining_size_bits / 32;
            if cfg!(test) { println!("size of incomplete block in word = {}", l_rest_size_word); }

            // Check if there are remaining datas to copy
            // Check if there are less than 512 data bits to copy
            if cfg!(test) { println!("Check if there are less than 512 data bits to copy"); }
            if (block_index * 512 + u64::from(remaining_size_bits)) == size_bit {
                // Copy partial block
                if cfg!(test) { println!("Copy partial block"); }
                // Work at byte level because number of remaining byte can be different from 4 multiple
                for byte_index in 0..(remaining_size_bits / 8) {
                    working_block[(byte_index / 4) as usize] |= u32::from(data[byte_index as usize]) << (24 - 8 * (byte_index % 4 ));
                }
                if cfg!(test) { display_block(&working_block); }
            }

            let mut reset_word_start_index = 0;
            let mut reset_word_end_index = 16;
            // Check if we are in the block where to put the additional 1 bit
            if cfg!(test) { println!("Check if we are in the block where to put the additional 1 bit"); }
            if (size_bit  / 512) == block_index {
                // Setting one additional bit to 1
                if cfg!(test) {
                    println!("Setting additional bit to 1");
                    println!("Index of block where to set the additional bit to 1 = {}", remaining_size_bits / 32);
                    println!("Shifting of {}", 31u32.wrapping_sub(remaining_size_bits));
                    println!("Mask = 0x{:X}", 1u32 << ((31u32.wrapping_sub(remaining_size_bits) % 32)));
                }
                working_block[(remaining_size_bits / 32) as usize] |=  1u32 << ((31u32.wrapping_sub(remaining_size_bits) % 32));

                if cfg!(test) { display_block(&working_block); }

                reset_word_start_index = 1 + ((remaining_size_bits + 2) / 32);
            }

            // Adding the size
            // Check if we are in the latest block
            if block_index + 1 == nb_blocks {
                // Adding the size on 64 bits
                working_block[14] = (size_bit >> u32::BITS) as u32;
                working_block[15] = (size_bit & u64::from(u32::MAX)) as u32;
                reset_word_end_index = 14;
            }

            // Setting rest of word to 0
            for word_index in reset_word_start_index..reset_word_end_index {
                if cfg!(test) { println!("Setting word[{}] to 0", word_index); }
                working_block[word_index as usize] = 0;
            }
        }

        let mut words: [u32; 80] = [0; 80];

        if cfg!(test) {
            // Display block to treat
            //---------------------------
            display_block(&working_block);
        }

        // Initialising word array
        //----------------------------
        // The 16 first words are the data themself
        words[0..16].copy_from_slice(&working_block[0..16]);

        // Computing the other words
        for word_index in 16..words.len() {
            if cfg!(test) { println!("Computing Word[{}]", word_index); }
            words[word_index] = u32::rotate_left(words[word_index - 3] ^ words[word_index - 8] ^ words[word_index - 14] ^ words[word_index - 16], 1);
        }

        //display words
        if cfg!(test) {
            for (index, word) in words.iter().enumerate() {
                println!("Word[{}] = 0x{:08X}", index, word);
            }
        }

        // Initialising variables
        //-------------------------
        let (mut a, mut b, mut c, mut d, mut e) = (key[0], key[1], key[2], key[3], key[4]);

        // Performing the "round"
        //-------------------------
        let constants: [u32; 4] = [ 0x5a827999 // 0 to 19
                                  , 0x6ed9eba1 //20 to 39
                                  , 0x8f1bbcdc //40 to 59
                                  , 0xca62c1d6  //60 to 79
                                  ];

        for round_index in 0..words.len() {
            let temp = u32::rotate_left(a, 5).wrapping_add(f(b, c, d, round_index as u32))
                                             .wrapping_add(e)
                                             .wrapping_add(constants[round_index / 20])
                                             .wrapping_add(words[round_index]);
            e = d;
            d = c;
            c = u32::rotate_left(b,30);
            b = a;
            a = temp;
        }

        // Computing intermediate hash value
        key[0] = key[0].wrapping_add(a);
        key[1] = key[1].wrapping_add(b);
        key[2] = key[2].wrapping_add(c);
        key[3] = key[3].wrapping_add(d);
        key[4] = key[4].wrapping_add(e);
    }
    Sha1Key::from_array(key)
}

#[cfg(test)]
mod test {
    use super::*;

    impl Sha1Key {
        fn new(word0: u32, word1: u32, word2: u32, word3: u32, word4: u32) -> Sha1Key {
            Sha1Key {words: [word0, word1, word2, word3, word4]}
        }
    }

    #[test]
    fn test_sha1_key_compare() {
        let key_ref = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x4);
        let key_other = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x5);
        assert_ne!(key_ref, key_other);
    }
    #[test]
    fn test_sha1_key_display() {
        let key_ref = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x4);
        assert_eq!(format!("{}", key_ref), "0000000400000003000000020000000100000000");
    }
    #[test]
    fn test_sha1_key_from_array() {
        let key_ref = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x4);
        let key_array = Sha1Key::from_array([0x0, 0x1, 0x2, 0x3, 0x4]);
        assert_eq!(key_ref, key_array);
    }
    #[test]
    fn test_sha1_string() {
        let key_ref = Sha1Key::new( 0x86f7e437
                                  , 0xfaa5a7fc
                                  , 0xe15d1ddc
                                  , 0xb9eaeaea
                                  , 0x377667b8
                                  );
        let data = vec!('a' as u8);
        assert_eq!(key_ref, compute_sha1(data));
    }
    #[test]
    fn test_sha1_string2() {
        let key_ref = Sha1Key::new( 0x7b502c3a
                                  , 0x1f48c860
                                  , 0x9ae212cd
                                  , 0xfb639dee
                                  , 0x39673f5e
                                  );
        let data: Vec<u8> = Vec::from("Hello world");
        assert_eq!(key_ref, compute_sha1(data));
    }
    #[test]
    fn test_sha1_string3() {
        let key_ref = Sha1Key::new( 0xa9993e36
                                  , 0x4706816a
                                  , 0xba3e2571
                                  , 0x7850c26c
                                  , 0x9cd0d89d
                                  );
        let data: Vec<u8> = Vec::from("abc");
        assert_eq!(key_ref, compute_sha1(data));
    }
    #[test]
    fn test_sha1_string4() {
        let key_ref = Sha1Key::new( 0x84983e44
                                  , 0x1c3bd26e
                                  , 0xbaae4aa1
                                  , 0xf95129e5
                                  , 0xe54670f1
                                  );
        let data: Vec<u8> = Vec::from("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(key_ref, compute_sha1(data));
    }
    #[test]
    fn test_sha1_string5() {
        let key_ref = Sha1Key::new( 0xe0c094e8
                                  , 0x67ef46c3
                                  , 0x50ef54a7
                                  , 0xf59dd60b
                                  , 0xed92ae83
                                  );
        let data: Vec<u8> = Vec::from("0123456701234567012345670123456701234567012345670123456701234567");
        assert_eq!(key_ref, compute_sha1(data));
    }
    #[test]
    fn test_sha1_string6() {
        let key_ref = Sha1Key::new( 0xda39a3ee
                                  , 0x5e6b4b0d
                                  , 0x3255bfef
                                  , 0x95601890
                                  , 0xafd80709
                                  );
        let data: Vec<u8> = Vec::from("");
        assert_eq!(key_ref, compute_sha1(data));
    }

}
