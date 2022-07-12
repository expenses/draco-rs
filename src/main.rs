fn main() {
    let filename = std::env::args().nth(1).unwrap();

    let mut bytes = std::fs::read(&filename).unwrap();

    let mut bytes: &[u8] = &bytes;

    let header = Header::parse(&bytes[..Header::LENGTH].try_into().unwrap());

    bytes = &bytes[Header::LENGTH..];

    dbg!(&header);

    let (indices, conn_dat) = decode_connectivity_data(&header, &mut bytes);

    decode_attribute_data(&header, &conn_dat, &mut bytes);
}

#[derive(Debug, PartialEq)]
enum EncoderMethod {
    MeshSequential,
    MeshEdgebreaker
}

impl EncoderMethod {
    fn parse(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::MeshSequential),
            1 => Some(Self::MeshEdgebreaker),
            _ => None
        }
    }
}

#[derive(Debug)]
struct Header {
    major_version: u8,
    minor_version: u8,
    encoder_type: u8,
    encoder_method: EncoderMethod,
    flags: u16,
}

impl Header {
    const LENGTH: usize = 11;

    fn parse(bytes: &[u8; Self::LENGTH]) -> Self {
        if &bytes[0..5] != b"DRACO" {
            panic!();
        }

        Self {
            major_version: bytes[5],
            minor_version: bytes[6],
            encoder_type: bytes[7],
            encoder_method: EncoderMethod::parse(bytes[8]).unwrap(),
            flags: bytes_to_u16(&bytes[9..])
        }
    }
}

fn decode_connectivity_data(header: &Header, data: &mut &[u8]) -> (Vec<[u16; 3]>, SequentialConnectivityData) {
    if header.encoder_method == EncoderMethod::MeshSequential {
        decode_sequential_connectivity_data(header, data)
    } else {
        todo!()
    }
}

fn decode_sequential_connectivity_data(header: &Header, bytes: &mut &[u8]) -> (Vec<[u16; 3]>, SequentialConnectivityData) {
    let data = SequentialConnectivityData::parse(bytes);

    dbg!(&data);

    if data.connectivity_method == SequentialIndicesEncodingMethod::Compressed {
        todo!()
    } else {
        (decode_sequential_indices(&data, bytes), data)
    }
}

#[derive(Debug)]
struct SequentialConnectivityData {
    num_faces: u32,
    num_points: u32,
    connectivity_method: SequentialIndicesEncodingMethod
}

impl SequentialConnectivityData {
    fn parse(bytes: &mut &[u8]) -> Self {
        Self {
            num_faces: leb128::read::unsigned(bytes).unwrap() as u32,
            num_points: leb128::read::unsigned(bytes).unwrap() as u32,
            connectivity_method: SequentialIndicesEncodingMethod::parse(bytes[0]).unwrap()
        }
    }
}

#[derive(Debug, PartialEq)]
enum SequentialIndicesEncodingMethod {
    Compressed,
    Uncompressed
}

impl SequentialIndicesEncodingMethod {
    fn parse(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Compressed),
            1 => Some(Self::Uncompressed),
            _ => None
        }
    }
}

fn decode_sequential_indices(data: &SequentialConnectivityData, bytes: &mut &[u8]) -> Vec<[u16; 3]> {
    if data.num_points < 256 {
        todo!()
    } else if (data.num_points < (1  << 16)) {
        parse_sequential_indices_u16(data, bytes)
    } else {
        todo!()
    }
}

fn parse_sequential_indices_u16(data: &SequentialConnectivityData, bytes: &mut &[u8]) -> Vec<[u16; 3]> {
    let mut indices = Vec::new();
    
    for i in 0 .. data.num_faces {
        indices.push([
            bytes_to_u16(&bytes[0..2]),
            bytes_to_u16(&bytes[2..4]),
            bytes_to_u16(&bytes[4..6]),
        ]);
        *bytes = &bytes[6..];
    }

    indices
} 

fn decode_attribute_data(header: &Header, conn_dat: &SequentialConnectivityData, bytes: &mut &[u8]) {
    let data = AttributeDecodersData::parse(header, bytes);

    dbg!(&data.attributes.len());

    let vertex_visited_point_ids = vec![0; data.attributes.len()];

    let mut curr_att_dec = 0;

    if header.encoder_method == EncoderMethod::MeshEdgebreaker {
        todo!();
    }

    let mut encoded_attribute_value_index_to_corner_map = vec![vec![]; data.attributes.len()];

    for i in 0 .. data.attributes.len() {
        curr_att_dec = i;

        let mut is_face_visited = vec![false; conn_dat.num_faces as usize];
        let mut is_vertex_visited = vec![false; conn_dat.num_faces as usize * 3];

        encoded_attribute_value_index_to_corner_map[curr_att_dec] = vec![0; conn_dat.num_points as usize];

        generate_sequence(header, conn_dat, curr_att_dec, &mut encoded_attribute_value_index_to_corner_map);
    }

    let mut att_dec_num_values_to_decode = vec![vec![]; data.attributes.len()];

    for i in 0 .. data.attributes.len() {
        att_dec_num_values_to_decode[i] = vec![0; data.attributes[i].attributes.len()];

        for j in 0..data.attributes[i].attributes.len() {
            att_dec_num_values_to_decode[i][j] = encoded_attribute_value_index_to_corner_map[i].len();
        }
    }

    for i in 0 .. data.attributes.len() {
        curr_att_dec = i;

        decode_portable_attributes(&data.attributes[i], bytes, curr_att_dec);
    }
}

fn decode_portable_attributes(attributes: &Attributes, bytes: &mut &[u8], curr_att_dec: usize) {
    for i in 0 .. attributes.attributes.len() {
        let prediction_scheme = bytes[0];
        dbg!(&prediction_scheme);
        *bytes = &bytes[1..];
    }
}

struct PredictionData {
    prediction_scheme: u8,

}

fn generate_sequence(header: &Header, conn_dat: &SequentialConnectivityData, curr_att_dec: usize, encoded_attribute_value_index_to_corner_map: &mut Vec<Vec<u32>>) {
    if header.encoder_method == EncoderMethod::MeshEdgebreaker {
        todo!()
    }  else {
        sequential_generate_sequence(conn_dat, curr_att_dec, encoded_attribute_value_index_to_corner_map)
    }
}

fn sequential_generate_sequence(conn_dat: &SequentialConnectivityData, curr_att_dec: usize, encoded_attribute_value_index_to_corner_map: &mut Vec<Vec<u32>>) {
    for i in 0 .. conn_dat.num_points {
        encoded_attribute_value_index_to_corner_map[curr_att_dec as usize][i as usize] = i;
    }
}

#[derive(Debug)]
struct AttributeDecodersData {
    attribute_decoders: Option<Vec<AttributeDecoder>>,
    attributes: Vec<Attributes>,

}

impl AttributeDecodersData {
    fn parse(header: &Header, bytes: &mut &[u8]) -> Self {
        let num_attribute_decoders = bytes[0];

        Self {
            attribute_decoders: if header.encoder_method == EncoderMethod::MeshEdgebreaker {
                panic!()
            } else {
                None
            },
            attributes: (0..num_attribute_decoders).map(|_| Attributes::parse(bytes)).collect()
        } 
    }
}

#[derive(Debug)]
struct Attributes {
    attributes: Vec<Attribute>,
    decoded_types: Vec<u8>,
}

impl Attributes {
    fn parse(bytes: &mut &[u8]) -> Self {
        let num_attributes = leb128::read::unsigned(bytes).unwrap() as usize;

        Self {
            attributes: (0..num_attributes).map(|_| Attribute::parse(bytes)).collect(),
            decoded_types: {
                let decoder_types = bytes[0..num_attributes].to_vec();

                *bytes = &bytes[num_attributes..];

                decoder_types
            }
        }
    }
}

#[derive(Debug)]
struct Attribute {
    att_type: u8,
    data_type: u8,
    num_components: u8,
    normalized: u8,
    dec_unique_id: u32
}

impl Attribute {
    fn parse(bytes: &mut &[u8]) -> Self {
        Self {
            att_type: bytes[0],
            data_type: bytes[1],
            num_components: bytes[2],
            normalized: bytes[3],
            dec_unique_id: leb128::read::unsigned(&mut &bytes[4..]).unwrap() as u32
        }
    }
}

#[derive(Debug)]
struct AttributeDecoder {
    data_id: u8,
    decoder_type: u8,
    traversal_method: u8,
}

impl AttributeDecoder {
    fn parse(header: &Header, bytes: &mut &[u8]) -> Self {
        let this = Self {
            data_id: bytes[0],
            decoder_type: bytes[1],
            traversal_method: bytes[2],
        };

        *bytes = &bytes[3..];

        this
    }
}

fn bytes_to_u16(bytes: &[u8]) -> u16 {
    let v = u16::from_le_bytes(
        bytes
            .get(0..2)
            .unwrap()
            .try_into()
            .unwrap(),
    );
    v
}