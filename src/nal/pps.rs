use crate::rbsp::BitRead;
use crate::{rbsp, Context};

// TODO: this is unchanged from original H264 parser, so it is completely incorrect for H265

#[derive(Debug)]
pub enum PpsError {
    RbspReaderError(rbsp::BitReaderError),
    InvalidSliceGroupMapType(u32),
    InvalidNumSliceGroupsMinus1(u32),
    InvalidNumRefIdx(&'static str, u32),
    InvalidSliceGroupChangeType(u32),
    UnknownSeqParamSetId(ParamSetId<15>),
    BadPicParamSetId(ParamSetIdError),
    BadSeqParamSetId(ParamSetIdError),
}

impl From<rbsp::BitReaderError> for PpsError {
    fn from(e: rbsp::BitReaderError) -> Self {
        PpsError::RbspReaderError(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParamSetIdError {
    IdTooLarge(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParamSetId<const MAX: u32>(u8);
impl<const MAX: u32> ParamSetId<MAX> {
    pub fn from_u32(id: u32) -> Result<Self, ParamSetIdError> {
        if id > MAX {
            Err(ParamSetIdError::IdTooLarge(id))
        } else {
            Ok(Self(id as u8))
        }
    }
    pub fn id(self) -> u8 {
        self.0
    }
}

pub type PicParamSetId = ParamSetId<63>;
pub type SeqParamSetId = ParamSetId<15>;

#[derive(Clone, Debug)]
pub struct PicParameterSet {
    pub pic_parameter_set_id: PicParamSetId,
    pub seq_parameter_set_id: SeqParamSetId,
    // TODO...
}
impl PicParameterSet {
    pub fn from_bits<R: BitRead>(_ctx: &Context, mut _r: R) -> Result<PicParameterSet, PpsError> {
        unimplemented!("Not implemented yet");
    }
}

// TODO: tests are not updated for H265
#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::*;

    #[test]
    fn test_it() {
        let data = hex!(
            "64 00 0A AC 72 84 44 26 84 00 00
            00 04 00 00 00 CA 3C 48 96 11 80"
        );
        let sps = super::sps::SeqParameterSet::from_bits(rbsp::BitReader::new(&data[..]))
            .expect("unexpected test data");
        let mut ctx = Context::default();
        ctx.put_seq_param_set(sps);
        let data = hex!("E8 43 8F 13 21 30");
        match PicParameterSet::from_bits(&ctx, rbsp::BitReader::new(&data[..])) {
            Err(e) => panic!("failed: {:?}", e),
            Ok(pps) => {
                println!("pps: {:#?}", pps);
                assert_eq!(pps.pic_parameter_set_id.id(), 0);
                assert_eq!(pps.seq_parameter_set_id.id(), 0);
            }
        }
    }

    #[test]
    fn test_transform_8x8_mode_with_scaling_matrix() {
        let sps = hex!(
            "64 00 29 ac 1b 1a 50 1e 00 89 f9 70 11 00 00 03 e9 00 00 bb 80 e2 60 00 04 c3 7a 00 00
             72 70 e8 c4 b8 c4 c0 00 09 86 f4 00 00 e4 e1 d1 89 70 f8 e1 85 2c"
        );
        let pps = hex!(
            "ea 8d ce 50 94 8d 18 b2 5a 55 28 4a 46 8c 59 2d 2a 50 c9 1a 31 64 b4 aa 85 48 d2 75 d5
             25 1d 23 49 d2 7a 23 74 93 7a 49 be 95 da ad d5 3d 7a 6b 54 22 9a 4e 93 d6 ea 9f a4 ee
             aa fd 6e bf f5 f7"
        );
        let sps = super::sps::SeqParameterSet::from_bits(rbsp::BitReader::new(&sps[..]))
            .expect("unexpected test data");
        let mut ctx = Context::default();
        ctx.put_seq_param_set(sps);

        let pps = PicParameterSet::from_bits(&ctx, rbsp::BitReader::new(&pps[..]))
            .expect("we mis-parsed pic_scaling_matrix when transform_8x8_mode_flag is active");

        // if transform_8x8_mode_flag were false or pic_scaling_matrix were None then we wouldn't
        // be recreating the required conditions for the test
        assert!(matches!(
            pps.extension,
            Some(PicParameterSetExtra {
                transform_8x8_mode_flag: true,
                pic_scaling_matrix: Some(_),
                ..
            })
        ));
    }
}
