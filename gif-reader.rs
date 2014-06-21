use std::comm;
use std::io::{File, Open, Read};

#[deriving(Show)]
enum Version {
  Undefined,
  Version87a,
  Version89a,
}
struct GifReader {
  version: Version,
  width: u16,
  height: u16,
  color_table: [u32, ..256],
  color_count: uint,
  background_color_index: u8,
  frame_sender: Sender<GifFrame>
}

impl GifReader {
    fn ReadHeader(&mut self, source: &mut Reader ) {
        let signature = source.read_exact(3).unwrap();
        // TODO: Use a byte literal (rustc 0.11)
        if signature != ~['G' as u8, 'I' as u8, 'F' as u8] {
            fail!("GIF signature missing");
        }
        
        let version = source.read_exact(3).unwrap();
        if version == ~['8' as u8, '7' as u8, 'a' as u8] {
          self.version = Version87a;
        } else if version == ~['8' as u8, '9' as u8, 'a' as u8] {
          self.version = Version89a;
        } else {
          fail!("Unrecognized version");
        }
        
        self.width = source.read_le_u16().unwrap();
        self.height = source.read_le_u16().unwrap();
        
        println!("width = {}", self.width);
        println!("height = {}", self.height);
        
        let packed = source.read_u8().unwrap();
        let has_global_color_table = (packed & 0xf0) != 0;
        let color_resolution = (packed >> 4) & 0x03;
        let sort_flag = (packed & 0xf0) != 0;
        self.background_color_index = source.read_u8().unwrap();
        let pixel_aspect_ratio = source.read_u8().unwrap();
        
        self.color_count = 2 << (packed & 7);
        
        println!("has_global_color_table = {}", has_global_color_table);
        println!("sort_flag = {}", sort_flag);
        println!("color_count = {}", self.color_count);
        println!("color_resolution = {}", color_resolution);
        println!("pixel_aspect_ratio = {}", pixel_aspect_ratio);
        
        
        if has_global_color_table {
            self.ReadColorTable(source);
        }
        
        for x in self.color_table.iter().take(self.color_count) {
            println!("color = {}", x)
        }
    }
  
    fn ReadColorTable(&mut self, source: &mut Reader) {
        let table_bytes = source.read_exact(self.color_count * 3).unwrap();
        for i in range(0, self.color_count) {
            self.color_table[i] =
                (table_bytes[i*3] as u32)
              | ((table_bytes[i*3+1] as u32) << 8) 
              | ((table_bytes[i*3+2] as u32) << 16);
        }
    }
    
    fn Read(&mut self, source: &mut Reader ) {
        self.ReadHeader(source);
    }
}

struct GifController {
    frame_receiver: Receiver<GifFrame>
}

fn init_gif_reader() -> (~GifController, ~GifReader) {
    let (frame_sender, frame_receiver): (Sender<GifFrame>, Receiver<GifFrame>) = comm::channel();
    let controller = ~GifController{
        frame_receiver: frame_receiver,
    };
    let reader = ~GifReader{
        version: Undefined,
        width: 0,
        height: 0,
        frame_sender: frame_sender,
        color_table: [0, ..256],
        color_count: 0,
        background_color_index: 0,
    };
    return (controller, reader)
}

struct GifFrame {
    width: int,
    height: int,
    pixels: ~[u32]
}


fn main() {
    let (_, worker) = init_gif_reader();
    
    spawn(proc() {
        let mut myworker = *worker;
        let p = Path::new("./sample_1.gif");

        let mut file = match File::open_mode(&p, Open, Read) {
            Ok(f) => f,
            Err(e) => fail!("file error: {}", e),
        };
        
        myworker.Read(&mut file);
    });
}
