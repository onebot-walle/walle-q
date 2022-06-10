use prost::Message;
use ricq_core::pb::msg::Ptt;
use walle_core::extended_map;
use walle_core::resp::FileIdContent;

pub trait SVoice: Sized {
    fn get_md5(&self) -> &[u8];
    fn get_size(&self) -> u32;
    fn to_data(&self) -> Vec<u8>;
    fn from_data(data: Vec<u8>) -> Self;
    fn voice_id(&self) -> Vec<u8> {
        [self.get_md5(), self.get_size().to_be_bytes().as_slice()].concat()
    }
    fn hex_voice_id(&self) -> String {
        hex::encode(self.voice_id())
    }
    fn as_file_id_content(&self) -> FileIdContent {
        FileIdContent {
            file_id: self.hex_voice_id(),
            extra: extended_map! {},
        }
    }
}

impl SVoice for Ptt {
    fn get_md5(&self) -> &[u8] {
        self.file_md5()
    }
    fn get_size(&self) -> u32 {
        self.file_size() as u32
    }
    fn to_data(&self) -> Vec<u8> {
        self.encode_to_vec()
    }
    fn from_data(data: Vec<u8>) -> Self {
        Message::decode(&data[..]).unwrap()
    }
}
