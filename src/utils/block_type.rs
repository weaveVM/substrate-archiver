use {
    borsh::{from_slice, to_vec},
    borsh_derive::{BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    std::io::{Read, Write},
};

#[derive(Debug, Deserialize, Serialize, BorshDeserialize, BorshSerialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    wvm_data: String,
}

impl Block {
    pub fn new(wvm_data: String) -> Self {
        Block { wvm_data }
    }
}

impl Block {
    pub fn brotli_compress(input: &[u8]) -> Vec<u8> {
        let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
        writer.write_all(input).unwrap();
        writer.into_inner()
    }

    pub fn brotli_decompress(input: Vec<u8>) -> Vec<u8> {
        let mut decompressed_data = Vec::new();
        let mut decompressor = brotli::Decompressor::new(input.as_slice(), 4096);
        decompressor
            .read_to_end(&mut decompressed_data)
            .expect("Decompression failed");
        decompressed_data
    }

    pub fn borsh_ser(input: &Block) -> Vec<u8> {
        to_vec(input).unwrap()
    }

    pub fn borsh_der(input: Vec<u8>) -> Block {
        from_slice(&input).expect("Error deserializing the calldata")
    }
}
