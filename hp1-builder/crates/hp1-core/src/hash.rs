use crate::CoreError;
use std::io::Read;
use std::path::Path;

pub fn sha256_file(path: &Path) -> Result<String, CoreError> {
    let mut file = std::fs::File::open(path)?;
    let mut state = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop { let count = file.read(&mut buffer)?; if count == 0 { break; } state.update(&buffer[..count]); }
    Ok(state.finish().iter().map(|byte| format!("{byte:02x}")).collect())
}

struct Sha256 { state: [u32; 8], buffer: [u8; 64], len: u64, used: usize }
impl Sha256 {
    fn new() -> Self { Self { state: [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19], buffer: [0;64], len: 0, used: 0 } }
    fn update(&mut self, input: &[u8]) { for &byte in input { self.buffer[self.used] = byte; self.used += 1; self.len += 8; if self.used == 64 { self.block(); self.used = 0; } } }
    fn finish(mut self) -> [u8; 32] { self.buffer[self.used] = 0x80; self.used += 1; if self.used > 56 { for index in self.used..64 { self.buffer[index] = 0; } self.block(); self.used = 0; } for index in self.used..56 { self.buffer[index] = 0; } self.buffer[56..64].copy_from_slice(&self.len.to_be_bytes()); self.block(); let mut result = [0;32]; for (index, word) in self.state.iter().enumerate() { result[index*4..index*4+4].copy_from_slice(&word.to_be_bytes()); } result }
    fn block(&mut self) { const K: [u32;64] = [0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2]; let mut w=[0u32;64]; for i in 0..16 { w[i]=u32::from_be_bytes(self.buffer[i*4..i*4+4].try_into().unwrap()); } for i in 16..64 { let s0=w[i-15].rotate_right(7)^w[i-15].rotate_right(18)^(w[i-15]>>3); let s1=w[i-2].rotate_right(17)^w[i-2].rotate_right(19)^(w[i-2]>>10); w[i]=w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1); } let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut h)=(self.state[0],self.state[1],self.state[2],self.state[3],self.state[4],self.state[5],self.state[6],self.state[7]); for i in 0..64 { let s1=e.rotate_right(6)^e.rotate_right(11)^e.rotate_right(25); let choice=(e&f)^((!e)&g); let t1=h.wrapping_add(s1).wrapping_add(choice).wrapping_add(K[i]).wrapping_add(w[i]); let s0=a.rotate_right(2)^a.rotate_right(13)^a.rotate_right(22); let majority=(a&b)^(a&c)^(b&c); let t2=s0.wrapping_add(majority); h=g;g=f;f=e;e=d.wrapping_add(t1);d=c;c=b;b=a;a=t1.wrapping_add(t2); } self.state[0]=self.state[0].wrapping_add(a);self.state[1]=self.state[1].wrapping_add(b);self.state[2]=self.state[2].wrapping_add(c);self.state[3]=self.state[3].wrapping_add(d);self.state[4]=self.state[4].wrapping_add(e);self.state[5]=self.state[5].wrapping_add(f);self.state[6]=self.state[6].wrapping_add(g);self.state[7]=self.state[7].wrapping_add(h); }
}
#[cfg(test)] mod tests { use super::*; #[test] fn sha256_vectors() { let mut sha=Sha256::new(); sha.update(b"abc"); assert_eq!(sha.finish().iter().map(|byte| format!("{byte:02x}")).collect::<String>(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"); } }
