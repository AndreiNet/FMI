use sha2::{Sha256, Digest};
use rayon::prelude::*;
use hex_literal::hex;

fn write_bytes(v: u32, target: &mut [u8]) {
    target[0] = (v & 0xff) as u8;
    target[1] = ((v >> 8) & 0xff) as u8;
    target[2] = ((v >> 16) & 0xff) as u8;
    target[3] = ((v >> 24) & 0xff) as u8;
}

#[derive(Clone)]
struct Block {
    version: u32,
    prev_block: [u8; 32],
    merkle_root: [u8; 32],
    time: u32,
    bits: u32,
    nonce: u32,
}

// Samples
impl Block {
    fn get_block_1() -> Block {
        Block {
            version: 0x20400000,
            prev_block: hex!("00000000000000000006a4a234288a44e715275f1775b77b2fddb6c02eb6b72f"),
            merkle_root: hex!("2dc60c563da5368e0668b81bc4d8dd369639a1134f68e425a9a74e428801e5b8"),
            time: 0x5DB8AB5E,
            bits: 0x17148EDF,
            nonce: 3_000_000_000,
        }
    }

    fn get_block_2() -> Block {
        Block {
            version: 0x20000000,
            prev_block: hex!("00000000000000000004cbd9f6080b5940664ca29c6c0939ce80a130d5271500"),
            merkle_root: hex!("8589836e206661ade5f5fa3c142630053a9bb192d823832e99620c6d16919043"),
            time: 1586852508,
            bits: 0x171320bc,
            nonce: 0x9853eba8,
        }
    }
}

impl Block {
    fn compute_hash(&self) -> String {
        self.compute_hash_with_nonce(self.nonce)
    }

    fn compute_hash_with_nonce(&self, nonce: u32) -> String {
        let block_len = 32;
        let buffer_lenth = 4 * 4 + 2 * block_len;
        let mut buffer = vec![0u8; buffer_lenth];
        write_bytes(self.version, &mut buffer);
        write_bytes(self.time, &mut buffer[(4 + 2 * block_len)..]);
        write_bytes(self.bits, &mut buffer[(4 + 2 * block_len + 4)..]);
        write_bytes(nonce, &mut buffer[(4 + 2 * block_len + 8)..]);
        buffer[4..(4 + block_len)].copy_from_slice(&self.prev_block);
        buffer[4..(4 + block_len)].reverse();
        buffer[(4 + block_len)..(4 + 2 * block_len)].copy_from_slice(&self.merkle_root);
        buffer[(4 + block_len)..(4 + 2 * block_len)].reverse();
        let block_hash = Sha256::digest(&buffer);
        let mut block_hash = Sha256::digest(&block_hash);
        block_hash.reverse();
        hex::encode(block_hash)
        
    }

    fn target(&self) -> String {
        let exp = self.bits >> 24;
        let mantissa = self.bits & 0xfffffff;
        let mut result = "00".repeat((exp - 3) as usize);
        result = format!("{:x}", mantissa) + &result;
        if result.len() < 64 {
            result = "0".repeat(64 - result.len()) + &result;
        }
        result
    }
}

fn mine1_par() {
    let block = Block::get_block_1();
    let target = block.target();
    let steps = 100_000_000u32;
    let step = (0..steps)
        .into_par_iter()
        .find_first(|step| {
            block.compute_hash_with_nonce(block.nonce + step) < target
        });
    let nonce = block.nonce + step.expect("Not found");
    println!("Nonce: {}", nonce);
}

fn mine2_par() {
    let mut block = Block::get_block_2();
    let target = block.target();
    block.nonce = 3060331852;
    let steps = 100_000_000u32;
    let first_nonce = block.nonce + 2489138;
    let step = (0..steps)
        .into_par_iter()
        .find_first(|step| {
            block.compute_hash_with_nonce(first_nonce + step) < target
        });
    match step {
        Some(s) => println!("Found: {}", s + first_nonce),
        None => println!("Not found"),
    }
}

fn check() {
    let mut block = Block::get_block_1();
    println!("{}", block.compute_hash_with_nonce(3060331852));
}

fn first5() {
    let block = Block::get_block_1();
    for i in block.nonce..(block.nonce + 5) {
        println!("{}", block.compute_hash_with_nonce(i));
    }
}

fn main() {
    // mine1_par();
    // check();
    // first5();
    // mine2_par();
}
