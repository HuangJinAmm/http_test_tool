use crypto::aes::{cbc_decryptor, cbc_encryptor, ctr, ecb_decryptor, ecb_encryptor, KeySize};
use crypto::blockmodes::PkcsPadding;
use crypto::buffer::{RefReadBuffer, RefWriteBuffer, WriteBuffer};
use crypto::symmetriccipher::{Decryptor, Encryptor};

fn get_key_size(key:&[u8]) -> Result<KeySize,String> {
	match key.len() {
		16 => Ok(KeySize::KeySize128),
		24 => Ok(KeySize::KeySize192),
		32 => Ok(KeySize::KeySize256),
		_ => return Err("Invalid key size (should be 128/192/256)".to_string()),
	}
}


pub fn aes_enc_ecb_string(key:&str,input: &str) ->Result<Vec<u8>,String> {
    let input = input.as_bytes();
    let key_bytes =  key.as_bytes();
    let key_size = get_key_size(key_bytes)?;
    aes_enc_ecb(key_size, key_bytes, input)
}

fn aes_enc_ecb(key_size: KeySize, key: &[u8], input: &[u8]) -> Result<Vec<u8>, String> {
	let mut a = ecb_encryptor(key_size, key, PkcsPadding);
	let cipher_len = cipher_length(input.len());
	let mut result = vec![0u8; cipher_len];
	a.encrypt(
		&mut RefReadBuffer::new(&input),
		&mut RefWriteBuffer::new(&mut result),
		true,
	)
	.map_err(|_| "Enc failed")?;
	Ok(result)
}

pub fn aes_enc_cbc_string(key:&str,input: &str,iv:&str) ->Result<Vec<u8>,String> {
    let input = input.as_bytes();
    let key_bytes =  key.as_bytes();
    let iv = iv.as_bytes();
    let key_size = get_key_size(key_bytes)?;
    aes_enc_cbc(key_size, key_bytes, input, iv)
}

fn aes_enc_cbc(key_size: KeySize, key: &[u8], input: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
	let mut a = cbc_encryptor(key_size, key, iv, PkcsPadding);
	let cipher_len = cipher_length(input.len());
	let mut result = vec![0u8; cipher_len];
	a.encrypt(
		&mut RefReadBuffer::new(&input),
		&mut RefWriteBuffer::new(&mut result),
		true,
	)
	.map_err(|_| "Enc failed")?;
	Ok(result)
}

pub fn aes_enc_ctr_string(key:&str,input: &str,iv:&str) -> Result<Vec<u8>,String> {
    let input = input.as_bytes();
    let key_bytes = key.as_bytes();
    let iv = iv.as_bytes();
    let key_size = get_key_size(key_bytes)?;
    aes_enc_ctr(key_size, key_bytes, input, iv)
}

fn aes_enc_ctr(key_size: KeySize, key: &[u8], input: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
	let mut a = ctr(key_size, key, iv);
	let mut result = vec![0u8; input.len()];
	a.encrypt(
		&mut RefReadBuffer::new(&input),
		&mut RefWriteBuffer::new(&mut result),
		true,
	)
	.map_err(|_| "Enc failed")?;
	Ok(result)
}

pub fn aes_dec_ecb_string(key:&str,input: &str) ->Result<Vec<u8>,String> {
    let input = base64::decode(input).map_err(|e|e.to_string())?;
    let key_bytes =  key.as_bytes();
    let key_size = get_key_size(key_bytes)?;
    aes_dec_ecb(key_size, key_bytes, input.as_slice())
}

fn aes_dec_ecb(key_size: KeySize, key: &[u8], input: &[u8]) -> Result<Vec<u8>, String> {
	let mut a = ecb_decryptor(key_size, key, PkcsPadding);
	let mut result = vec![0u8; input.len()];
	let mut buffer = RefWriteBuffer::new(&mut result);
	a.decrypt(&mut RefReadBuffer::new(&input), &mut buffer, true)
		.map_err(|_| "Dec failed")?;
	let len = buffer.capacity() - buffer.remaining();
	let mut result = result.clone();
	result.truncate(len);
	Ok(result)
}

pub fn aes_dec_cbc_string(key:&str,input: &str,iv:&str) ->Result<Vec<u8>,String> {
    let input = base64::decode(input).map_err(|e|e.to_string())?;
    let key_bytes =  key.as_bytes();
    let iv = iv.as_bytes();
    let key_size = get_key_size(key_bytes)?;
    aes_dec_cbc(key_size, key_bytes, input.as_slice(), iv)
}
fn aes_dec_cbc(key_size: KeySize, key: &[u8], input: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
	let mut a = cbc_decryptor(key_size, key, iv, PkcsPadding);
	let mut result = vec![0u8; input.len()];
	let mut buffer = RefWriteBuffer::new(&mut result);
	a.decrypt(&mut RefReadBuffer::new(&input), &mut buffer, true)
		.map_err(|_| "Dec failed")?;
	let len = buffer.capacity() - buffer.remaining();
	let mut result = result.clone();
	result.truncate(len);
	Ok(result)
}
pub fn aes_dec_ctr_string(key:&str,input: &str,iv:&str) -> Result<Vec<u8>,String> {
    let input = base64::decode(input).map_err(|e|e.to_string())?;
    let key_bytes = key.as_bytes();
    let iv = iv.as_bytes();
    let key_size = get_key_size(key_bytes)?;
    aes_dec_ctr(key_size, key_bytes, input.as_slice(), iv)
}


fn aes_dec_ctr(key_size: KeySize, key: &[u8], input: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
	let mut a = ctr(key_size, key, iv);
	let mut result = vec![0u8; input.len()];
	let mut buffer = RefWriteBuffer::new(&mut result);
	a.decrypt(&mut RefReadBuffer::new(&input), &mut buffer, true)
		.map_err(|_| "Dec failed")?;
	Ok(result)
}

const BLOCK_SIZE: usize = 16;

fn cipher_length(input_len: usize) -> usize {
	((input_len / BLOCK_SIZE) + 1) * BLOCK_SIZE
}