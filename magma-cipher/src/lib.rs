use hex_literal::hex;
use magma::cipher::generic_array::ArrayLength;
use magma::cipher::{
    generic_array::{typenum::U8, GenericArray},
    BlockDecrypt, BlockEncrypt, KeyInit,
};
use magma::Magma;
use rand::prelude::*;

use std::fs::{File, self};
use std::io::{Read, Write};
use std::os::raw;

enum Status {
    Encrypted,
    Decrypted,
}

#[no_mangle]
pub extern "C" fn encrypt(path: *const raw::c_char) {
    let path = unsafe {std::ffi::CStr::from_ptr(path).to_str().expect("Input string error") };
    let arr = bytes_from_file(&path.to_string());
    let key = generate_key();
    let arr = encrypt_array(arr, key.clone());
    bytes_to_file(&path.to_string(), arr, Status::Encrypted);
    key_to_file(key);
}

#[no_mangle]
pub extern "C" fn decrypt(path: *const raw::c_char) {
    let key = key_from_file();
    let path = unsafe {std::ffi::CStr::from_ptr(path).to_str().expect("Input string error") };
    let arr = bytes_from_file(&path.to_string());
    let arr = decrypt_array(arr, key);
    bytes_to_file(&path.to_string(), arr, Status::Decrypted);
}

fn generate_key() -> Vec<u8> {
    if let Ok(mut f) = File::open("key.txt") {
	let mut key = vec![0, 32];
	f.read(&mut key).expect("Can not read key.txt");
	key
    } else {
	let ret : [u8; 32] = [0; 32];
	let mut r = rand::thread_rng();
	ret.iter().map(|_| (r.next_u32() % 256) as u8).collect()
    }
}

fn key_from_file() -> Vec<u8> {
    let mut file = File::open("key.txt").expect("Can not open file");
    let mut key : Vec<u8> = vec![0; 32];
    dbg!(key.clone());
    file.read(&mut key).expect("Unable read file");
    key
}

fn key_to_file(key : Vec<u8>) {
    let mut file = File::create("key.txt").expect("Can not create file");
    file.write_all(key.as_slice()).unwrap();
}

fn encrypt_array(arr: Vec<u8>, key: Vec<u8>) -> Vec<u8> {
    let cipher = Magma::new(key.as_slice().into());
    let mut arr = arr;
    let reminder = (8 - (arr.len() % 8)) % 8;
    for _ in 0..reminder {
        arr.push(0);
    }
    let chunks: Vec<&[u8]> = arr.chunks(8).collect();
    let mut ret = Vec::new();
    for i in chunks.iter() {
        let mut block = GenericArray::from_slice(*i).clone();
	cipher.encrypt_block(&mut block);
	let mut block = block.to_vec();
	ret.append(&mut block);
    }
    ret
}

fn decrypt_array(arr: Vec<u8>, key: Vec<u8>) -> Vec<u8> {
    let cipher = Magma::new(key.as_slice().into());
    let  chunks: Vec<&[u8]> = arr.chunks(8).collect();
    let mut ret = Vec::new();
    for i in chunks.iter() {
        let mut block = GenericArray::clone_from_slice(*i);
	cipher.decrypt_block(&mut block);
	let mut block = block.to_vec();
	ret.append(&mut block);
    }
    while let Some(0) = ret.last() {
	ret.pop().unwrap();
    }
    ret
}

fn bytes_from_file(path: &String) -> Vec<u8> {
    let mut file = File::open(&path).expect("File not found!");
    let meta = fs::metadata(&path).expect("Unable to read file metadata");
    let mut buf = vec![0; meta.len() as usize];
    file.read(&mut buf).expect("Unable to read data");
    buf
}

fn bytes_to_file(path: &String, arr : Vec<u8>, status: Status) {
    let path = match status {
        Status::Encrypted => path.clone() + ".enc",
        Status::Decrypted => if path.ends_with(".enc") { path.as_str()[0..path.len() - 4].to_string() } else { path.clone() },
    };
    let mut file = File::create(&path).expect("File not found!");
    file.write_all(arr.as_slice()).expect("Can not write to file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_array() {
        let key = hex!("FFEEDDCCBBAA99887766554433221100"
		       "F0F1F2F3F4F5F6F7F8F9FAFBFCFDFEFF");
        let arr = hex!("FEDCBA9876543210");
        let ciph_arr: Vec<u8> = hex!("4EE901E5C2D8CA3D").into();
	
        let arr = encrypt_array(arr.into(), key.into());
        assert_eq!(arr, ciph_arr);
    }

    #[test]
    fn test_encrypt_key_improve() {
	let key = generate_key();
	let arr = hex!("ABABBCBCABBCBCABFD093019A4");
	let arr = encrypt_array(arr.into(), key);
	assert_eq!(arr.len(), 16);
    }

    #[test]
    fn test_decrypt_key_improve() {
	let key = generate_key();
	let arr = hex!("ABABBCBCABBCBCABFD093019A4");
	let arr = encrypt_array(arr.into(), key.clone());
	let arr = decrypt_array(arr, key);
	dbg!(arr.clone());
	assert_eq!(arr.len(), 13);
    }
    
    #[test]
    fn test_decrypt_array() {
	let key = hex!("FFEEDDCCBBAA99887766554433221100"
		       "F0F1F2F3F4F5F6F7F8F9FAFBFCFDFEFF");
        let arr : Vec<u8> = hex!("FEDCBA9876543210").into();
        let ciph_arr = hex!("4EE901E5C2D8CA3D");

        let ciph_arr = decrypt_array(ciph_arr.into(), key.into());
        assert_eq!(arr, ciph_arr);
    }

    #[test]
    fn test_generation() {
	let key = generate_key();
	assert_eq!(key.len(), 32)
    }
}
