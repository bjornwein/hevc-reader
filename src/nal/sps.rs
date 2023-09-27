use crate::{
    nal::pps::{ParamSetId, ParamSetIdError},
    rbsp::{BitRead, BitReaderError},
};
use std::fmt::Debug;

// TODO: more really specific errors after adding more constraints...
#[derive(Debug)]
pub enum SpsError {
    /// Signals that bit_depth_luma_minus8 was greater than the max value, 6
    // BitDepthOutOfRange(u32),
    RbspReaderError(BitReaderError),
    /// log2_max_frame_num_minus4 must be between 0 and 12
    // Log2MaxFrameNumMinus4OutOfRange(u32),
    BadSeqParamSetId(ParamSetIdError),
    BadVideoParamSetId(ParamSetIdError),
    /// A field in the bitstream had a value too large for a subsequent calculation
    FieldValueTooLarge {
        name: &'static str,
        value: u32,
    },
    /// The `cpb_cnt_minus1` field must be between 0 and 31 inclusive.
    // CpbCountOutOfRange(u32),

    /// An unimplemented part of the SPS syntax was encountered
    /// TODO: These errors should be removed before serious release
    Unimplemented(&'static str),
}

impl From<BitReaderError> for SpsError {
    fn from(e: BitReaderError) -> Self {
        SpsError::RbspReaderError(e)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Main,
    High,
}
impl Tier {
    // TODO: understand semantics better
    pub fn from_tier_flag(flag: bool) -> Tier {
        match flag {
            false => Tier::Main,
            true => Tier::High,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Unknown(u8),

    // Main profile
    Main,

    // Main 10 and Main 10 Still Picture profiles
    Main10,
    Main10StillPicture,

    // Main Still Picture profile
    MainStillPicture,

    // Format range extensions profiles
    Monochrome,
    Monochrome10,
    Monochrome12,
    Monochrome16,
    Main12,
    Main422_10,
    Main422_12,
    Main444,
    Main444_10,
    Main444_12,
    MainIntra,
    Main10Intra,
    Main12Intra,
    Main422_10Intra,
    Main422_12Intra,
    Main444Intra,
    Main444_10Intra,
    Main444_12Intra,
    Main444_16Intra,
    Main444StillPicture,
    Main444_16StillPicture,

    // High throughput profiles
    HighThroughput444,
    HighThroughput444_10,
    HighThroughput444_14,
    HighThroughput444_16Intra,

    // Screen content coding extensions profiles
    ScreenExtendedMain,
    ScreenExtendedMain10,
    ScreenExtendedMain444,
    ScreenExtendedMain444_10,

    // High throughput screen content coding extensions profiles
    ScreenExtendedHighThroughput444,
    ScreenExtendedHighThroughput444_10,
    ScreenExtendedHighThroughput444_14,

    // Scalable Main and Scalable Main 10 profiles
    ScalableMain,
    ScalableMain10,

    // Scalable format range extensions profiles
    ScalableMonochrome,
    ScalableMonochrome12,
    ScalableMonochrome16,
    ScalableMain444,

    // Multiview Main profile
    MultiviewMain,

    // 3D Main profile
    ThreeDeeMain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    L1,
    L2,
    L2_1,
    L3,
    L3_1,
    L4,
    L4_1,
    L5,
    L5_1,
    L5_2,
    L6,
    L6_1,
    L6_2,

    L8_5,

    /// Note that the value carried is the idc value, which is 30x level
    Reserved(u8),
}

impl Level {
    pub fn from_level_idc(level_idc: u8) -> Level {
        // level_idc is 30 * level
        match level_idc {
            30 => Level::L1,
            60 => Level::L2,
            63 => Level::L2_1,
            90 => Level::L3,
            93 => Level::L3_1,
            120 => Level::L4,
            123 => Level::L4_1,
            150 => Level::L5,
            153 => Level::L5_1,
            156 => Level::L5_2,
            180 => Level::L6,
            183 => Level::L6_1,
            186 => Level::L6_2,
            255 => Level::L8_5,
            n => Level::Reserved(n),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChromaFormat {
    Monochrome,
    #[default]
    YUV420,
    YUV422,
    YUV444,
    Invalid(u32),
}
impl ChromaFormat {
    fn from_chroma_format_idc(chroma_format_idc: u32) -> ChromaFormat {
        match chroma_format_idc {
            0 => ChromaFormat::Monochrome,
            1 => ChromaFormat::YUV420,
            2 => ChromaFormat::YUV422,
            3 => ChromaFormat::YUV444,
            _ => ChromaFormat::Invalid(chroma_format_idc),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromaInfo {
    pub chroma_format: ChromaFormat,
    pub separate_colour_plane_flag: bool,
}
impl ChromaInfo {
    pub fn read<R: BitRead>(r: &mut R) -> Result<ChromaInfo, SpsError> {
        let chroma_format_idc = r.read_ue("chroma_format_idc")?;
        Ok(ChromaInfo {
            chroma_format: ChromaFormat::from_chroma_format_idc(chroma_format_idc),
            separate_colour_plane_flag: if chroma_format_idc == 3 {
                r.read_bool("separate_colour_plane_flag")?
            } else {
                false
            },
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AspectRatioInfo {
    #[default]
    Unspecified,
    Ratio1_1,
    Ratio12_11,
    Ratio10_11,
    Ratio16_11,
    Ratio40_33,
    Ratio24_11,
    Ratio20_11,
    Ratio32_11,
    Ratio80_33,
    Ratio18_11,
    Ratio15_11,
    Ratio64_33,
    Ratio160_99,
    Ratio4_3,
    Ratio3_2,
    Ratio2_1,
    Reserved(u8),
    Extended(u16, u16),
}
impl AspectRatioInfo {
    fn read<R: BitRead>(r: &mut R) -> Result<Option<AspectRatioInfo>, BitReaderError> {
        let aspect_ratio_info_present_flag = r.read_bool("aspect_ratio_info_present_flag")?;
        Ok(if aspect_ratio_info_present_flag {
            let aspect_ratio_idc = r.read_u8(8, "aspect_ratio_idc")?;
            Some(match aspect_ratio_idc {
                0 => AspectRatioInfo::Unspecified,
                1 => AspectRatioInfo::Ratio1_1,
                2 => AspectRatioInfo::Ratio12_11,
                3 => AspectRatioInfo::Ratio10_11,
                4 => AspectRatioInfo::Ratio16_11,
                5 => AspectRatioInfo::Ratio40_33,
                6 => AspectRatioInfo::Ratio24_11,
                7 => AspectRatioInfo::Ratio20_11,
                8 => AspectRatioInfo::Ratio32_11,
                9 => AspectRatioInfo::Ratio80_33,
                10 => AspectRatioInfo::Ratio18_11,
                11 => AspectRatioInfo::Ratio15_11,
                12 => AspectRatioInfo::Ratio64_33,
                13 => AspectRatioInfo::Ratio160_99,
                14 => AspectRatioInfo::Ratio4_3,
                15 => AspectRatioInfo::Ratio3_2,
                16 => AspectRatioInfo::Ratio2_1,
                255 => AspectRatioInfo::Extended(
                    r.read_u16(16, "sar_width")?,
                    r.read_u16(16, "sar_height")?,
                ),
                _ => AspectRatioInfo::Reserved(aspect_ratio_idc),
            })
        } else {
            None
        })
    }

    /// Returns the aspect ratio as `(width, height)`, if specified.
    pub fn get(&self) -> Option<(u16, u16)> {
        match self {
            AspectRatioInfo::Unspecified => None,
            AspectRatioInfo::Ratio1_1 => Some((1, 1)),
            AspectRatioInfo::Ratio12_11 => Some((12, 11)),
            AspectRatioInfo::Ratio10_11 => Some((10, 11)),
            AspectRatioInfo::Ratio16_11 => Some((16, 11)),
            AspectRatioInfo::Ratio40_33 => Some((40, 33)),
            AspectRatioInfo::Ratio24_11 => Some((24, 11)),
            AspectRatioInfo::Ratio20_11 => Some((20, 11)),
            AspectRatioInfo::Ratio32_11 => Some((32, 11)),
            AspectRatioInfo::Ratio80_33 => Some((80, 33)),
            AspectRatioInfo::Ratio18_11 => Some((18, 11)),
            AspectRatioInfo::Ratio15_11 => Some((15, 11)),
            AspectRatioInfo::Ratio64_33 => Some((64, 33)),
            AspectRatioInfo::Ratio160_99 => Some((160, 99)),
            AspectRatioInfo::Ratio4_3 => Some((4, 3)),
            AspectRatioInfo::Ratio3_2 => Some((3, 2)),
            AspectRatioInfo::Ratio2_1 => Some((2, 1)),
            AspectRatioInfo::Reserved(_) => None,
            &AspectRatioInfo::Extended(width, height) => {
                // ISO/IEC 14496-10 section E.2.1: "When ... sar_width is equal to 0 or sar_height
                // is equal to 0, the sample aspect ratio shall be considered unspecified by this
                // Recommendation | International Standard."
                if width == 0 || height == 0 {
                    None
                } else {
                    Some((width, height))
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum OverscanAppropriate {
    #[default]
    Unspecified,
    Appropriate,
    Inappropriate,
}
impl OverscanAppropriate {
    fn read<R: BitRead>(r: &mut R) -> Result<OverscanAppropriate, BitReaderError> {
        let overscan_info_present_flag = r.read_bool("overscan_info_present_flag")?;
        Ok(if overscan_info_present_flag {
            let overscan_appropriate_flag = r.read_bool("overscan_appropriate_flag")?;
            if overscan_appropriate_flag {
                OverscanAppropriate::Appropriate
            } else {
                OverscanAppropriate::Inappropriate
            }
        } else {
            OverscanAppropriate::Unspecified
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum VideoFormat {
    #[default]
    Component,
    PAL,
    NTSC,
    SECAM,
    MAC,
    Unspecified,
    Reserved(u8),
}
impl VideoFormat {
    fn from(video_format: u8) -> VideoFormat {
        match video_format {
            0 => VideoFormat::Component,
            1 => VideoFormat::PAL,
            2 => VideoFormat::NTSC,
            3 => VideoFormat::SECAM,
            4 => VideoFormat::MAC,
            5 => VideoFormat::Unspecified,
            6 | 7 => VideoFormat::Reserved(video_format),
            // This shouldn't be possible considering the single use of this function.
            _ => panic!("unsupported video_format value {}", video_format),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColourDescription {
    pub colour_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coeffs: u8,
}
impl ColourDescription {
    fn read<R: BitRead>(r: &mut R) -> Result<Option<ColourDescription>, BitReaderError> {
        let colour_description_present_flag = r.read_bool("colour_description_present_flag")?;
        Ok(if colour_description_present_flag {
            Some(ColourDescription {
                colour_primaries: r.read_u8(8, "colour_primaries")?,
                transfer_characteristics: r.read_u8(8, "transfer_characteristics")?,
                matrix_coeffs: r.read_u8(8, "matrix_coeffs")?,
            })
        } else {
            None
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VideoSignalType {
    pub video_format: VideoFormat,
    pub video_full_range_flag: bool,
    pub colour_description: Option<ColourDescription>,
}
impl VideoSignalType {
    fn read<R: BitRead>(r: &mut R) -> Result<Option<VideoSignalType>, BitReaderError> {
        let video_signal_type_present_flag = r.read_bool("video_signal_type_present_flag")?;
        Ok(if video_signal_type_present_flag {
            Some(VideoSignalType {
                video_format: VideoFormat::from(r.read_u8(3, "video_format")?),
                video_full_range_flag: r.read_bool("video_full_range_flag")?,
                colour_description: ColourDescription::read(r)?,
            })
        } else {
            None
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChromaLocInfo {
    pub chroma_sample_loc_type_top_field: u32,
    pub chroma_sample_loc_type_bottom_field: u32,
}
impl ChromaLocInfo {
    fn read<R: BitRead>(r: &mut R) -> Result<Option<ChromaLocInfo>, BitReaderError> {
        let chroma_loc_info_present_flag = r.read_bool("chroma_loc_info_present_flag")?;
        Ok(if chroma_loc_info_present_flag {
            Some(ChromaLocInfo {
                chroma_sample_loc_type_top_field: r.read_ue("chroma_sample_loc_type_top_field")?,
                chroma_sample_loc_type_bottom_field: r
                    .read_ue("chroma_sample_loc_type_bottom_field")?,
            })
        } else {
            None
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub win_left_offset: u32,
    pub win_right_offset: u32,
    pub win_top_offset: u32,
    pub win_bottom_offset: u32,
}
impl Window {
    pub fn read<R: BitRead>(r: &mut R) -> Result<Option<Self>, SpsError> {
        Ok(if r.read_bool("window_flag")? {
            Some(Self {
                win_left_offset: r.read_ue("win_left_offset")?,
                win_right_offset: r.read_ue("win_right_offset")?,
                win_top_offset: r.read_ue("win_top_offset")?,
                win_bottom_offset: r.read_ue("win_bottom_offset")?,
            })
        } else {
            None
        })
    }
}

// TODO: Check if this is generalizable with Vui && Vps
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimingInfo {
    pub num_units_in_tick: u32,
    pub time_scale: u32,
    pub num_ticks_poc_diff_one_minus1: Option<u32>,
    pub hrd_parameters: Option<HrdParameters>,
}
impl TimingInfo {
    fn read<R: BitRead>(
        r: &mut R,
        hrd_common_inf_present: bool,
        max_sub_layers_minus1: u8,
    ) -> Result<Option<TimingInfo>, BitReaderError> {
        let timing_info_present_flag = r.read_bool("timing_info_present_flag")?;
        Ok(if timing_info_present_flag {
            Some(TimingInfo {
                num_units_in_tick: r.read_u32(32, "num_units_in_tick")?,
                time_scale: r.read_u32(32, "time_scale")?,
                num_ticks_poc_diff_one_minus1: Self::read_num_ticks(r)?,
                hrd_parameters: HrdParameters::read(
                    r,
                    hrd_common_inf_present,
                    max_sub_layers_minus1,
                )?,
            })
        } else {
            None
        })
    }

    fn read_num_ticks<R: BitRead>(r: &mut R) -> Result<Option<u32>, BitReaderError> {
        let vui_poc_proprtional_timing_flag = r.read_bool("vui_poc_proprtional_timing_flag")?;
        Ok(if vui_poc_proprtional_timing_flag {
            Some(r.read_ue("vui_num_ticks_poc_diff_one_minus1")?)
        } else {
            None
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SubPicHrdParams {
    pub tick_divisor_minus2: u8,
    pub du_cpb_removal_delay_increment_length_minus1: u8,
    pub sub_pic_cpb_params_in_pic_timing_sei_flag: bool,
    pub dpb_output_delay_du_length_minus1: u8,
    pub cpb_size_du_scale: u8,
}
impl SubPicHrdParams {
    fn read_partial<R: BitRead>(r: &mut R) -> Result<Self, BitReaderError> {
        Ok(Self {
            tick_divisor_minus2: r.read_u8(8, "tick_divisor_minus2")?,
            du_cpb_removal_delay_increment_length_minus1: r
                .read_u8(5, "du_cpb_removal_delay_increment_length_minus1")?,
            sub_pic_cpb_params_in_pic_timing_sei_flag: r
                .read_bool("sub_pic_cpb_params_in_pic_timing_sei_flag")?,
            dpb_output_delay_du_length_minus1: r.read_u8(5, "dpb_output_delay_du_length_minus1")?,
            cpb_size_du_scale: 0, // To be filled in later
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HrdParametersCommonInfParameters {
    pub sub_pic_hrd_params: Option<SubPicHrdParams>,
    pub bit_rate_scale: u8,
    pub cpb_size_scale: u8,
    pub initial_cpb_removal_delay_length_minus1: u8,
    pub au_cpb_removal_delay_length_minus1: u8,
    pub dpb_output_delay_length_minus1: u8,
}
impl HrdParametersCommonInfParameters {
    fn read<R: BitRead>(r: &mut R) -> Result<Self, BitReaderError> {
        let sub_pic_hrd_params_present = r.read_bool("sub_pic_hrd_params_present_flag")?;
        let mut sub_pic_hrd_params = if sub_pic_hrd_params_present {
            Some(SubPicHrdParams::read_partial(r)?)
        } else {
            None
        };
        let bit_rate_scale = r.read_u8(4, "bit_rate_scale")?;
        let cpb_size_scale = r.read_u8(4, "cpb_size_scale")?;
        if let Some(subpic) = sub_pic_hrd_params.as_mut() {
            subpic.cpb_size_du_scale = r.read_u8(4, "cpb_size_du_scale")?;
        }
        Ok(Self {
            sub_pic_hrd_params,
            bit_rate_scale,
            cpb_size_scale,
            initial_cpb_removal_delay_length_minus1: r
                .read_u8(5, "initial_cpb_removal_delay_length_minus1")?,
            au_cpb_removal_delay_length_minus1: r
                .read_u8(5, "au_cpb_removal_delay_length_minus1")?,
            dpb_output_delay_length_minus1: r.read_u8(5, "dpb_output_delay_length_minus1")?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HrdParametersCommonInf {
    pub nal_hrd_parameters_present_flag: bool,
    pub vcl_hrd_parameters_present_flag: bool,
    pub parameters: Option<HrdParametersCommonInfParameters>,
}
impl HrdParametersCommonInf {
    fn read<R: BitRead>(r: &mut R) -> Result<Self, BitReaderError> {
        let nal_hrd_parameters_present_flag = r.read_bool("nal_hrd_parameters_present")?;
        let vcl_hrd_parameters_present_flag = r.read_bool("vcl_hrd_parameters_present")?;
        Ok(Self {
            nal_hrd_parameters_present_flag,
            vcl_hrd_parameters_present_flag,
            parameters: if nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag {
                Some(HrdParametersCommonInfParameters::read(r)?)
            } else {
                None
            },
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SubLayerSubPicHrdParams {
    pub cpb_size_du_value_minus1: u32,
    pub bit_rate_du_value_minus1: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SubLayerHrdParameter {
    pub bit_rate_value_minus1: u32,
    pub cpb_size_value_minus1: u32,
    pub sub_pic_hrd_params: Option<SubLayerSubPicHrdParams>,
    pub cbr_flag: bool,
}
impl SubLayerHrdParameter {
    fn read<R: BitRead>(
        r: &mut R,
        sub_pic_hrd_params_present: bool,
    ) -> Result<Self, BitReaderError> {
        Ok(SubLayerHrdParameter {
            bit_rate_value_minus1: r.read_ue("bit_rate_value_minus1")?,
            cpb_size_value_minus1: r.read_ue("cpb_size_value_minus1")?,
            sub_pic_hrd_params: if sub_pic_hrd_params_present {
                Some(SubLayerSubPicHrdParams {
                    cpb_size_du_value_minus1: r.read_ue("cpb_size_du_value_minus1")?,
                    bit_rate_du_value_minus1: r.read_ue("bit_rate_du_value_minus1")?,
                })
            } else {
                None
            },
            cbr_flag: r.read_bool("cbr_flag")?,
        })
    }
}

// The syntax here is a bit messy, so intial version doesn't
// split optional fields in subtypes. Make better types if needed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerHrdParameters {
    pub fixed_pic_rate_general_flag: bool,
    pub fixed_pic_rate_within_cvs_flag: bool, // valid iff !fixed_pic_rate_general_flag
    pub elemental_duration_in_tc_minus1: u32, // valid iff fixed_pic_rate_within_cvs_flag
    pub low_delay_hrd_flag: bool,             // valid iff !fixed_pic_rate_within_cvs_flag
    pub cpb_cnt_minus1: u32,                  // valid iff !low_delay_hrd_flag
    pub nal_hrd_parameters: Option<Vec<SubLayerHrdParameter>>,
    pub vcl_hrd_parameters: Option<Vec<SubLayerHrdParameter>>,
}
impl LayerHrdParameters {
    fn read<R: BitRead>(
        r: &mut R,
        nal_hrd_parameters_present: bool,
        vcl_hrd_parameters_present: bool,
        sub_pic_hrd_parameters_present: bool,
    ) -> Result<Self, BitReaderError> {
        let fixed_pic_rate_general_flag = r.read_bool("fixed_pic_rate_general_flag")?;
        let fixed_pic_rate_within_cvs_flag = if !fixed_pic_rate_general_flag {
            r.read_bool("fixed_pic_rate_within_cvs_flag")?
        } else {
            false // TODO default?
        };
        let (elemental_duration_in_tc_minus1, low_delay_hrd_flag) =
            if fixed_pic_rate_within_cvs_flag {
                (r.read_ue("elemental_duration_in_tc_minus1")?, false)
            } else {
                (0, r.read_bool("low_delay_hrd_flag")?)
            };
        let cpb_cnt_minus1 = if low_delay_hrd_flag {
            r.read_ue("cpb_cnt_minus1")?
        } else {
            0
        };
        // TODO: default value for cpb_cnt_minus1? (ie if low_delay_hrd_flag)
        let nal_hrd_parameters = if nal_hrd_parameters_present {
            let params: Result<Vec<_>, _> = (0..=cpb_cnt_minus1)
                .map(|_| SubLayerHrdParameter::read(r, sub_pic_hrd_parameters_present))
                .collect();
            Some(params?)
        } else {
            None
        };
        let vcl_hrd_parameters = if vcl_hrd_parameters_present {
            let params: Result<Vec<_>, _> = (0..=cpb_cnt_minus1)
                .map(|_| SubLayerHrdParameter::read(r, sub_pic_hrd_parameters_present))
                .collect();
            Some(params?)
        } else {
            None
        };

        Ok(LayerHrdParameters {
            fixed_pic_rate_general_flag,
            fixed_pic_rate_within_cvs_flag,
            elemental_duration_in_tc_minus1,
            low_delay_hrd_flag,
            cpb_cnt_minus1,
            nal_hrd_parameters,
            vcl_hrd_parameters,
        })
    }
}

// TODO: most or all vecs can be replace with ArrayVec to reduce allocations and indirections
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HrdParameters {
    pub common: Option<HrdParametersCommonInf>,
    pub layers: Vec<LayerHrdParameters>,
}
impl HrdParameters {
    fn read<R: BitRead>(
        r: &mut R,
        common_inf_present_flag: bool,
        max_num_sub_layers_minus1: u8,
    ) -> Result<Option<Self>, BitReaderError> {
        let hrd_parameters_present_flag = r.read_bool("hrd_parameters_present_flag")?;
        Ok(if hrd_parameters_present_flag {
            let common = if common_inf_present_flag {
                Some(HrdParametersCommonInf::read(r)?)
            } else {
                None
            };
            let mut layers = Vec::with_capacity(usize::from(max_num_sub_layers_minus1) + 1);
            let nal_hrd_params = common
                .as_ref()
                .map_or(false, |c| c.nal_hrd_parameters_present_flag);
            let vcl_hrd_params = common
                .as_ref()
                .map_or(false, |c| c.vcl_hrd_parameters_present_flag);
            let sub_pic_hrd_params = common
                .as_ref()
                .and_then(|c| c.parameters.as_ref())
                .map(|p| p.sub_pic_hrd_params.is_some())
                .unwrap_or(false);
            for _ in 0..=max_num_sub_layers_minus1 {
                layers.push(LayerHrdParameters::read(
                    r,
                    nal_hrd_params,
                    vcl_hrd_params,
                    sub_pic_hrd_params, // TODO: default values?
                )?);
            }
            Some(Self { common, layers })
        } else {
            None
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BitstreamRestrictions {
    pub tiles_fixed_structure_flag: bool,
    pub motion_vectors_over_pic_boundaries_flag: bool,
    pub restricted_ref_pic_lists_flag: bool,
    pub min_spatial_segmentation_idc: u32,
    pub max_bytes_per_pic_denom: u32,
    pub max_bits_per_mb_denom: u32,
    pub log2_max_mv_length_horizontal: u32,
    pub log2_max_mv_length_vertical: u32,
}
impl BitstreamRestrictions {
    fn read<R: BitRead>(r: &mut R) -> Result<Option<BitstreamRestrictions>, BitReaderError> {
        let bitstream_restriction_flag = r.read_bool("bitstream_restriction_flag")?;
        Ok(if bitstream_restriction_flag {
            Some(BitstreamRestrictions {
                tiles_fixed_structure_flag: r.read_bool("tiles_fixed_structure_flag")?,
                motion_vectors_over_pic_boundaries_flag: r
                    .read_bool("motion_vectors_over_pic_boundaries_flag")?,
                restricted_ref_pic_lists_flag: r.read_bool("restricted_ref_pic_lists_flag")?,
                min_spatial_segmentation_idc: r.read_ue("min_spatial_segmentation_idc")?,
                max_bytes_per_pic_denom: r.read_ue("max_bytes_per_pic_denom")?,
                max_bits_per_mb_denom: r.read_ue("max_bits_per_mb_denom")?,
                log2_max_mv_length_horizontal: r.read_ue("log2_max_mv_length_horizontal")?,
                log2_max_mv_length_vertical: r.read_ue("log2_max_mv_length_vertical")?,
            })
        } else {
            None
        })
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct LayerProfile {
    pub profile_space: u8,
    pub tier_flag: bool,
    pub profile_idc: u8,
    pub profile_compatibility_flag: [bool; 32],
    pub progressive_source_flag: bool,
    pub interlaced_source_flag: bool,
    pub non_packed_constraint_flag: bool,
    pub frame_only_constraint_flag: bool,

    // TODO: default values for optional flags?
    pub max_14bit_constraint_flag: bool,
    pub max_12bit_constraint_flag: bool,
    pub max_10bit_constraint_flag: bool,
    pub max_8bit_constraint_flag: bool,
    pub max_422chroma_constraint_flag: bool,
    pub max_420chroma_constraint_flag: bool,
    pub max_monochrome_constraint_flag: bool,
    pub intra_constraint_flag: bool,
    pub one_picture_only_constraint_flag: bool,

    pub lower_bit_rate_constraint_flag: bool,
    pub inbld_flag: bool,
}
impl LayerProfile {
    pub fn read<R: BitRead>(r: &mut R) -> Result<LayerProfile, SpsError> {
        let profile_space = r.read_u8(2, "profile_space")?;
        let tier_flag = r.read_bool("tier_flag")?;
        let profile_idc = r.read_u8(5, "profile_idc")?;
        // TODO: faster(?) to read as an u32, easier to follow spec as bit array
        let mut profile_compatibility_flag = [false; 32];
        for flag in profile_compatibility_flag.iter_mut() {
            *flag = r.read_bool("profile_compatibility_flag[j]")?;
        }

        let mut profile = LayerProfile {
            profile_space,
            tier_flag,
            profile_idc,
            profile_compatibility_flag,
            progressive_source_flag: r.read_bool("progressive_source_flag")?,
            interlaced_source_flag: r.read_bool("interlaced_source_flag")?,
            non_packed_constraint_flag: r.read_bool("non_packed_constraint_flag")?,
            frame_only_constraint_flag: r.read_bool("frame_only_constraint_flag")?,
            ..LayerProfile::default()
        };

        // Conditional syntax here closely follow the spec
        if profile_idc == 4
            || profile_compatibility_flag[4]
            || profile_idc == 5
            || profile_compatibility_flag[5]
            || profile_idc == 6
            || profile_compatibility_flag[6]
            || profile_idc == 7
            || profile_compatibility_flag[7]
            || profile_idc == 8
            || profile_compatibility_flag[8]
            || profile_idc == 9
            || profile_compatibility_flag[9]
            || profile_idc == 10
            || profile_compatibility_flag[10]
            || profile_idc == 11
            || profile_compatibility_flag[11]
        {
            profile.max_12bit_constraint_flag = r.read_bool("max_12bit_constraint_flag")?;
            profile.max_10bit_constraint_flag = r.read_bool("max_10bit_constraint_flag")?;
            profile.max_8bit_constraint_flag = r.read_bool("max_8bit_constraint_flag")?;
            profile.max_422chroma_constraint_flag = r.read_bool("max_422chroma_constraint_flag")?;
            profile.max_420chroma_constraint_flag = r.read_bool("max_420chroma_constraint_flag")?;
            profile.max_monochrome_constraint_flag =
                r.read_bool("max_monochrome_constraint_flag")?;
            profile.intra_constraint_flag = r.read_bool("intra_constraint_flag")?;
            profile.one_picture_only_constraint_flag =
                r.read_bool("one_picture_only_constraint_flag")?;
            profile.lower_bit_rate_constraint_flag =
                r.read_bool("lower_bit_rate_constraint_flag")?;
            if profile_idc == 5
                || profile_compatibility_flag[5]
                || profile_idc == 9
                || profile_compatibility_flag[9]
                || profile_idc == 10
                || profile_compatibility_flag[10]
                || profile_idc == 11
                || profile_compatibility_flag[11]
            {
                profile.max_14bit_constraint_flag = r.read_bool("max_14bit_constraint_flag")?;
                let _zero_bits = r.read_u32(32, "reserved_zero_33bits")?;
                let _zero_bits = r.read_u32(1, "reserved_zero_33bits")?;
            } else {
                let _zero_bits = r.read_u32(32, "reserved_zero_34bits")?;
                let _zero_bits = r.read_u32(2, "reserved_zero_34bits")?;
            }
        } else if profile_idc == 2 || profile_compatibility_flag[2] {
            let _zero_bits = r.read_u8(7, "reserved_zero_7bits")?; // TODO: check zeroness
            profile.one_picture_only_constraint_flag =
                r.read_bool("one_picture_only_constraint_flag")?;
            let _zero_bits = r.read_u32(32, "reserved_zero_35bits")?;
            let _zero_bits = r.read_u32(3, "reserved_zero_35bits")?;
        } else {
            let _zero_bits = r.read_u32(32, "reserved_zero_43bits")?;
            let _zero_bits = r.read_u32(11, "reserved_zero_43bits")?;
        }
        if profile_idc == 1
            || profile_compatibility_flag[1]
            || profile_idc == 2
            || profile_compatibility_flag[2]
            || profile_idc == 3
            || profile_compatibility_flag[3]
            || profile_idc == 4
            || profile_compatibility_flag[4]
            || profile_idc == 5
            || profile_compatibility_flag[5]
            || profile_idc == 9
            || profile_compatibility_flag[9]
            || profile_idc == 11
            || profile_compatibility_flag[11]
        {
            profile.inbld_flag = r.read_bool("inbld_flag")?;
        } else {
            let _zero_bit = r.read_bool("reserved_zero_bit")?;
        }

        Ok(profile)
    }

    pub fn tier(&self) -> Tier {
        Tier::from_tier_flag(self.tier_flag)
    }

    /// Return the "lowest" compatible profile
    // TODO: this returns the "lowest" profile indicated by any profile_compatibility_flag
    // but in reality a (sub)stream can conform to multiple profiles by setting multiple flags.
    pub fn profile(&self) -> Profile {
        use Profile::*;

        if self.profile_idc == 1 || self.profile_compatibility_flag[1] {
            Main
        } else if self.profile_idc == 2 || self.profile_compatibility_flag[2] {
            if self.one_picture_only_constraint_flag {
                Main10StillPicture
            } else {
                Main10
            }
        } else if self.profile_idc == 3 || self.profile_compatibility_flag[3] {
            MainStillPicture
        } else if self.profile_idc == 4 || self.profile_compatibility_flag[4] {
            match (
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 1, 1, 0, 0, 1) => Monochrome,
                (1, 1, 0, 1, 1, 1, 0, 0, 1) => Monochrome10,
                (1, 0, 0, 1, 1, 1, 0, 0, 1) => Monochrome12,
                (0, 0, 0, 1, 1, 1, 0, 0, 1) => Monochrome16,
                (1, 0, 0, 1, 1, 0, 0, 0, 1) => Main12,
                (1, 1, 0, 1, 0, 0, 0, 0, 1) => Main422_10,
                (1, 0, 0, 1, 0, 0, 0, 0, 1) => Main422_12,
                (1, 1, 1, 0, 0, 0, 0, 0, 1) => Main444,
                (1, 1, 0, 0, 0, 0, 0, 0, 1) => Main444_10,
                (1, 0, 0, 0, 0, 0, 0, 0, 1) => Main444_12,
                (1, 1, 1, 1, 1, 0, 1, 0, _) => MainIntra,
                (1, 1, 0, 1, 1, 0, 1, 0, _) => Main10Intra,
                (1, 0, 0, 1, 1, 0, 1, 0, _) => Main12Intra,
                (1, 1, 0, 1, 0, 0, 1, 0, _) => Main422_10Intra,
                (1, 0, 0, 1, 0, 0, 1, 0, _) => Main422_12Intra,
                (1, 1, 1, 0, 0, 0, 1, 0, _) => Main444Intra,
                (1, 1, 0, 0, 0, 0, 1, 0, _) => Main444_10Intra,
                (1, 0, 0, 0, 0, 0, 1, 0, _) => Main444_12Intra,
                (0, 0, 0, 0, 0, 0, 1, 0, _) => Main444_16Intra,
                (1, 1, 1, 0, 0, 0, 1, 1, _) => Main444StillPicture,
                (0, 0, 0, 0, 0, 0, 1, 1, _) => Main444_16StillPicture,

                _ => Unknown(self.profile_idc),
            }
        } else if self.profile_idc == 5 || self.profile_compatibility_flag[5] {
            match (
                self.max_14bit_constraint_flag as u8,
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 0, 0, 0, 0, 0, 1) => HighThroughput444,
                (1, 1, 1, 0, 0, 0, 0, 0, 0, 1) => HighThroughput444_10,
                (1, 0, 0, 0, 0, 0, 0, 0, 0, 1) => HighThroughput444_14,
                (0, 0, 0, 0, 0, 0, 0, 1, 0, _) => HighThroughput444_16Intra,

                _ => Unknown(self.profile_idc),
            }
        } else if self.profile_idc == 6 || self.profile_compatibility_flag[6] {
            match (
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 1, 0, 0, 0, 1) => MultiviewMain,
                _ => Unknown(self.profile_idc),
            }
        } else if self.profile_idc == 7 || self.profile_compatibility_flag[7] {
            match (
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 1, 0, 0, 0, 1) => ScalableMain,
                (1, 1, 0, 1, 1, 0, 0, 0, 1) => ScalableMain10,
                _ => Unknown(self.profile_idc),
            }
        } else if self.profile_idc == 8 || self.profile_compatibility_flag[8] {
            match (
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 1, 0, 0, 0, 1) => ThreeDeeMain,
                _ => Unknown(self.profile_idc),
            }
        } else if self.profile_idc == 9 || self.profile_compatibility_flag[9] {
            match (
                self.max_14bit_constraint_flag as u8,
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 1, 1, 0, 0, 0, 1) => ScreenExtendedMain,
                (1, 1, 1, 0, 1, 1, 0, 0, 0, 1) => ScreenExtendedMain10,
                (1, 1, 1, 1, 0, 0, 0, 0, 0, 1) => ScreenExtendedMain444,
                (1, 1, 1, 0, 0, 0, 0, 0, 0, 1) => ScreenExtendedMain444_10,

                _ => Unknown(self.profile_idc),
            }
        } else if self.profile_idc == 10 || self.profile_compatibility_flag[10] {
            match (
                self.max_14bit_constraint_flag as u8,
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 1, 1, 1, 0, 0, 1) => ScalableMonochrome,
                (1, 1, 0, 0, 1, 1, 1, 0, 0, 1) => ScalableMonochrome12,
                (0, 0, 0, 0, 1, 1, 1, 0, 0, 1) => ScalableMonochrome16,
                (1, 1, 1, 1, 0, 0, 0, 0, 0, 1) => ScalableMain444,

                _ => Unknown(self.profile_idc),
            }
        } else if self.profile_idc == 11 || self.profile_compatibility_flag[11] {
            match (
                self.max_14bit_constraint_flag as u8,
                self.max_12bit_constraint_flag as u8,
                self.max_10bit_constraint_flag as u8,
                self.max_8bit_constraint_flag as u8,
                self.max_422chroma_constraint_flag as u8,
                self.max_420chroma_constraint_flag as u8,
                self.max_monochrome_constraint_flag as u8,
                self.intra_constraint_flag as u8,
                self.one_picture_only_constraint_flag as u8,
                self.lower_bit_rate_constraint_flag as u8,
            ) {
                (1, 1, 1, 1, 0, 0, 0, 0, 0, 1) => ScreenExtendedHighThroughput444,
                (1, 1, 1, 0, 0, 0, 0, 0, 0, 1) => ScreenExtendedHighThroughput444_10,
                (1, 0, 0, 0, 0, 0, 0, 0, 0, 1) => ScreenExtendedHighThroughput444_14,

                _ => Unknown(self.profile_idc),
            }
        } else {
            Unknown(self.profile_idc)
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct SubLayerProfileLevel {
    pub profile: Option<LayerProfile>,
    pub level_idc: Option<u8>,
}
impl SubLayerProfileLevel {
    pub fn read<R: BitRead>(
        r: &mut R,
        profile_present: bool,
        level_present: bool,
    ) -> Result<SubLayerProfileLevel, SpsError> {
        let profile = match profile_present {
            true => Some(LayerProfile::read(r)?),
            false => None,
        };

        let level_idc = match level_present {
            true => Some(r.read_u8(8, "sub_layer_level_idc[i]")?),
            false => None,
        };

        Ok(SubLayerProfileLevel { profile, level_idc })
    }
}

// TODO: used in both vps and pps. break out to "common_syntax" module and add custom errors?
/// Profile, Tier and Level
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileTierLevel {
    pub general_profile: Option<LayerProfile>,
    pub general_level_idc: u8,
    pub sub_layers: [SubLayerProfileLevel; 7],
}
impl ProfileTierLevel {
    pub fn read<R: BitRead>(
        r: &mut R,
        profile_present_flag: bool,
        max_num_sub_layers_minus1: u8,
    ) -> Result<ProfileTierLevel, SpsError> {
        let general_profile = match profile_present_flag {
            true => Some(LayerProfile::read(r)?),
            false => None,
        };
        let general_level_idc = r.read_u8(8, "general_level_idc")?;

        // TODO: This could maybe be an assert for max speed?
        SeqParameterSet::validate_max_num_sub_layers_minus1(max_num_sub_layers_minus1)?;

        let mut sub_layer_profile_present_flag = [false; 7];
        let mut sub_layer_level_present_flag = [false; 7];
        for i in 0..max_num_sub_layers_minus1 {
            sub_layer_profile_present_flag[usize::from(i)] =
                r.read_bool("sub_layer_profile_present_flag[i]")?;
            sub_layer_level_present_flag[usize::from(i)] =
                r.read_bool("sub_layer_level_present_flag[i]")?;
        }
        if max_num_sub_layers_minus1 > 0 {
            for _ in max_num_sub_layers_minus1..8 {
                let _zero_bits = r.read_u8(2, "reserved_zero_2bits[i]")?;
            }
        }
        let mut sub_layers = std::array::from_fn(|_| SubLayerProfileLevel::default());
        for (i, layer) in sub_layers.iter_mut().enumerate() {
            *layer = SubLayerProfileLevel::read(
                r,
                sub_layer_profile_present_flag[i],
                sub_layer_level_present_flag[i],
            )?;
        }

        Ok(ProfileTierLevel {
            general_profile,
            general_level_idc,
            sub_layers,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerInfo {
    pub sps_max_dec_pic_buffering_minus1: u32,
    pub sps_max_num_reorder_pics: u32,
    pub sps_max_latency_increase_plus1: u32,
}
impl LayerInfo {
    pub fn read<R: BitRead>(
        r: &mut R,
        sps_max_sub_layers_minus1: u8,
    ) -> Result<Vec<LayerInfo>, SpsError> {
        SeqParameterSet::validate_max_num_sub_layers_minus1(sps_max_sub_layers_minus1)?;

        let sub_layer_ordering_info_present =
            r.read_bool("sps_sub_layer_ordering_info_present_flag")?;

        Ok(if sub_layer_ordering_info_present {
            let mut layers = Vec::with_capacity((sps_max_sub_layers_minus1 + 1).into());
            for _ in 0..=sps_max_sub_layers_minus1 {
                layers.push(Self::read_layer(r)?);
            }
            layers
        } else {
            vec![Self::read_layer(r)?] // NOTE: index is wrong if sps_max_sub_layers_minus1 > 0
        })
    }

    fn read_layer<R: BitRead>(r: &mut R) -> Result<LayerInfo, SpsError> {
        Ok(LayerInfo {
            sps_max_dec_pic_buffering_minus1: r.read_ue("sps_max_dec_pic_buffering_minus1")?,
            sps_max_num_reorder_pics: r.read_ue("sps_max_num_reorder_pics")?,
            sps_max_latency_increase_plus1: r.read_ue("sps_max_latency_increase_plus1")?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScalingList; // TODO: store list contents
impl ScalingList {
    pub fn read<R: BitRead>(r: &mut R) -> Result<Option<ScalingList>, SpsError> {
        Ok(if r.read_bool("scaling_list_enabled_flag")? {
            if r.read_bool("sps_scaling_list_data_present_flag")? {
                Some(Self::read_scaling_list(r)?)
            } else {
                Some(ScalingList) // Enabled but empty
            }
        } else {
            None // Not enabled
        })
    }

    fn read_scaling_list<R: BitRead>(r: &mut R) -> Result<ScalingList, SpsError> {
        for size_id in 0..4 {
            for _matrix_id in (0..6).step_by(if size_id == 3 { 3 } else { 1 }) {
                if !r.read_bool("scaling_list_pred_mode_flag")? {
                    let _scaling_list_pred_matrix_id_delta =
                        r.read_ue("scaling_list_pred_matrix_id_delta")?;
                } else {
                    let mut next_coef = 8;
                    let coef_num = 64.min(1 << (4 + (size_id << 1)));
                    if size_id > 1 {
                        let scaling_list_dc_coef_minus8 =
                            r.read_se("scaling_list_dc_coef_minus8")?;
                        next_coef = scaling_list_dc_coef_minus8 + 8;
                    }
                    for _ in 0..coef_num {
                        let scaling_list_delta_coef = r.read_se("scaling_list_delta_coef")?;
                        next_coef = (next_coef + scaling_list_delta_coef + 256) % 256;
                    }
                }
            }
        }
        Ok(ScalingList)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pcm {
    pub pcm_sample_bit_depth_luma_minus1: u8,
    pub pcm_sample_bit_depth_chroma_minus1: u8,
    pub log2_min_pcm_luma_coding_block_size_minus3: u32,
    pub log2_diff_max_min_pcm_luma_coding_block_size: u32,
    pub pcm_loop_filter_disabled: bool,
}

impl Pcm {
    pub fn read<R: BitRead>(r: &mut R) -> Result<Option<Pcm>, SpsError> {
        Ok(if r.read_bool("pcm_enabled_flag")? {
            Some(Pcm {
                pcm_sample_bit_depth_luma_minus1: r
                    .read_u8(4, "pcm_sample_bit_depth_luma_minus1")?,
                pcm_sample_bit_depth_chroma_minus1: r
                    .read_u8(4, "pcm_sample_bit_depth_chroma_minus1")?,
                log2_min_pcm_luma_coding_block_size_minus3: r
                    .read_ue("log2_min_pcm_luma_coding_block_size_minus3")?,
                log2_diff_max_min_pcm_luma_coding_block_size: r
                    .read_ue("log2_diff_max_min_pcm_luma_coding_block_size")?,
                pcm_loop_filter_disabled: r.read_bool("pcm_loop_filter_disabled")?,
            })
        } else {
            None // Not enabled
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortTermRefPicSet; // TODO: contents
impl ShortTermRefPicSet {
    fn read<R: BitRead>(
        r: &mut R,
        st_rps_idx: u32,
        num_short_term_ref_pic_sets: u32,
    ) -> Result<Self, SpsError> {
        let inter_ref_pic_set_prediction_flag = if st_rps_idx == 0 {
            false // TODO: default for i==0?
        } else {
            r.read_bool("inter_ref_pic_set_prediction_flag")?
        };
        if inter_ref_pic_set_prediction_flag {
            if st_rps_idx == num_short_term_ref_pic_sets {
                let _delta_idx_minus1 = r.read_ue("delta_idx_minus1")?;
            }
            let _delta_rps_sign = r.read_bool("delta_rps_sign")?;
            let _abs_delta_rps_minus1 = r.read_ue("abs_delta_rps_minus1")?;

            // RefRpsIdx = stRpsIdx − ( delta_idx_minus1 + 1 )
            // NumDeltaPocs[ stRpsIdx ] = NumNegativePics[ stRpsIdx ] + NumPositivePics[ stRpsIdx ]
            return Err(SpsError::Unimplemented("num_delta_pics[ref_rps_idx]"));
            /*
            for j in 0..=num_delta_pocs[ref_rps_idx] {
                let used_by_curr_pic_flag = r.read_bool("used_by_curr_pic_flag")?;
                if used_by_curr_pic_flag {
                    let _use_delta_flag = r.read_bool("use_delta_flag")?;
                }
            }
            */
        } else {
            let num_negative_pics = r.read_ue("num_negative_pics")?;
            let num_positive_pics = r.read_ue("num_positive_pics")?;
            for _ in 0..num_negative_pics {
                let _delta_poc_s0_minus1 = r.read_ue("delta_pic_s0_minus1")?;
                let _used_by_curr_pic_s0_flag = r.read_bool("used_by_curr_pic_s0_flag")?;
            }
            for _ in 0..num_positive_pics {
                let _delta_poc_s1_minus1 = r.read_ue("delta_pic_s1_minus1")?;
                let _used_by_curr_pic_s1_flag = r.read_bool("used_by_curr_pic_s1_flag")?;
            }
        }

        Ok(ShortTermRefPicSet)
    }

    pub fn read_with_count<R: BitRead>(r: &mut R) -> Result<Vec<Self>, SpsError> {
        let num = r.read_ue("num_short_term_ref_pic_sets")?;
        (0..num).map(|i| Self::read(r, i, num)).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LongTermRefPicSps; // TODO: store content
impl LongTermRefPicSps {
    fn read_one<R: BitRead>(r: &mut R) -> Result<Self, SpsError> {
        let _lt_ref_pic_pic_lsb_sps = r.read_ue("lt_ref_pic_pic_lsb_sps")?;
        let _used_by_curr_pic_lt_sps_flag = r.read_ue("used_by_curr_pic_lt_sps_flag")?;

        Ok(LongTermRefPicSps)
    }

    pub fn read<R: BitRead>(r: &mut R) -> Result<Option<Vec<Self>>, SpsError> {
        let present = r.read_bool("long_term_ref_pics_present_flag")?;
        if present {
            let num = r.read_ue("num_long_term_ref_pics_sps")?;
            let refs: Result<Vec<_>, _> = (0..num).map(|_| Self::read_one(r)).collect();
            Ok(Some(refs?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VuiParameters {
    pub aspect_ratio_info: Option<AspectRatioInfo>,
    pub overscan_appropriate: OverscanAppropriate,
    pub video_signal_type: Option<VideoSignalType>,
    pub chroma_loc_info: Option<ChromaLocInfo>,
    pub neutral_chroma_indication_flag: bool,
    pub field_seq_flag: bool,
    pub frame_field_info_present_flag: bool,
    pub default_display_window: Option<Window>,
    pub timing_info: Option<TimingInfo>,
    pub bitstream_restrictions: Option<BitstreamRestrictions>,
}
impl VuiParameters {
    fn read_one<R: BitRead>(
        r: &mut R,
        hrd_common_inf_present: bool,
        max_sub_layers_minus1: u8,
    ) -> Result<Self, SpsError> {
        Ok(VuiParameters {
            aspect_ratio_info: AspectRatioInfo::read(r)?,
            overscan_appropriate: OverscanAppropriate::read(r)?,
            video_signal_type: VideoSignalType::read(r)?,
            chroma_loc_info: ChromaLocInfo::read(r)?,
            neutral_chroma_indication_flag: r.read_bool("neutral_chroma_indication_flag")?,
            field_seq_flag: r.read_bool("field_seq_flag")?,
            frame_field_info_present_flag: r.read_bool("frame_field_info_present_flag")?,
            default_display_window: Window::read(r)?,
            timing_info: TimingInfo::read(r, hrd_common_inf_present, max_sub_layers_minus1)?,
            bitstream_restrictions: BitstreamRestrictions::read(r)?,
        })
    }

    pub fn read<R: BitRead>(
        r: &mut R,
        hrd_common_inf_present: bool,
        max_sub_layers_minus1: u8,
    ) -> Result<Option<Self>, SpsError> {
        Ok(if r.read_bool("vui_parameeters_present")? {
            Some(Self::read_one(
                r,
                hrd_common_inf_present,
                max_sub_layers_minus1,
            )?)
        } else {
            None
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpsExtension; // TODO: contents
impl SpsExtension {
    fn read<R: BitRead>(r: &mut R) -> Result<Option<Self>, SpsError> {
        Ok(if r.read_bool("sps_extension_present_flag")? {
            let sps_range_extension_flag = r.read_bool("sps_range_extension_flag")?;
            let sps_multilayer_extension_flag = r.read_bool("sps_multilayer_extension_flag")?;
            let sps_3d_extension_flag = r.read_bool("sps_3d_extension_flag")?;
            let sps_scc_extension_flag = r.read_bool("sps_scc_extension_flag")?;
            let sps_extension_4bits = r.read_u8(4, "sps_extension_4bits")?;

            // TODO
            if sps_range_extension_flag {
                return Err(SpsError::Unimplemented("sps_range_extension"));
            }
            if sps_multilayer_extension_flag {
                return Err(SpsError::Unimplemented("sps_multilayer_extension"));
            }
            if sps_3d_extension_flag {
                return Err(SpsError::Unimplemented("sps_3d_extension"));
            }
            if sps_scc_extension_flag {
                return Err(SpsError::Unimplemented("sps_scc_extension"));
            }
            if sps_extension_4bits != 0 {
                while r.has_more_rbsp_data("sps_extension_data_flag")? {
                    r.read_bool("sps_extension_data_flag")?;
                }
            }

            Some(SpsExtension)
        } else {
            None
        })
    }
}

pub type VideoParamSetId = ParamSetId<15>;
pub type SeqParamSetId = ParamSetId<15>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeqParameterSet {
    pub sps_video_parameter_set_id: VideoParamSetId,
    pub sps_max_sub_layers_minus1: u8,
    pub sps_temporal_id_nesting: bool,
    pub profile_tier_level: ProfileTierLevel,
    pub sps_seq_parameter_set_id: SeqParamSetId,
    pub chroma_info: ChromaInfo,
    pub pic_width_in_luma_samples: u32,
    pub pic_height_in_luma_samples: u32,
    pub conformance_window: Option<Window>,
    pub bit_depth_luma_minus8: u32,
    pub bit_depth_chroma_minus8: u32,
    pub log2_max_pic_order_cnt_lsb_minus4: u32,
    pub sub_layering_ordering_info: Vec<LayerInfo>,
    pub log2_min_luma_coding_block_size_minus3: u32,
    pub log2_diff_max_min_luma_coding_block_size: u32,
    pub log2_min_luma_transform_block_size_minus2: u32,
    pub log2_diff_max_min_luma_transform_block_size: u32,
    pub max_transform_hierarchy_depth_inter: u32,
    pub max_transform_hierarchy_depth_intra: u32,
    pub scaling_list: Option<ScalingList>,
    pub amp_enabled: bool,
    pub sample_adaptive_offset_enabled: bool,
    pub pcm: Option<Pcm>,
    pub st_ref_pic_sets: Vec<ShortTermRefPicSet>,
    pub long_term_ref_pics_sps: Option<Vec<LongTermRefPicSps>>,
    pub sps_termporal_mvp_enabled: bool,
    pub strong_intra_smoothing_enabled: bool,
    pub vui_parameters: Option<VuiParameters>,
    pub sps_extension: Option<SpsExtension>,
}
impl SeqParameterSet {
    pub fn from_bits<R: BitRead>(mut r: R) -> Result<SeqParameterSet, SpsError> {
        let sps_video_parameter_set_id = r.read_u8(4, "sps_video_parameter_set_id")?;
        let sps_max_sub_layers_minus1 = r.read_u8(3, "sps_max_sub_layers_minus1")?;

        // TODO: should apply more max/min validations to many of those parameters
        let sps = SeqParameterSet {
            sps_video_parameter_set_id: ParamSetId::from_u32(sps_video_parameter_set_id.into())
                .map_err(SpsError::BadVideoParamSetId)?,
            sps_max_sub_layers_minus1,
            sps_temporal_id_nesting: r.read_bool("sps_temporal_id_nesting_flag")?,

            profile_tier_level: ProfileTierLevel::read(&mut r, true, sps_max_sub_layers_minus1)?, // check

            sps_seq_parameter_set_id: ParamSetId::from_u32(r.read_ue("seq_parameter_set_id")?)
                .map_err(SpsError::BadSeqParamSetId)?,
            chroma_info: ChromaInfo::read(&mut r)?,
            pic_width_in_luma_samples: r.read_ue("pic_width_in_luma_samples")?,
            pic_height_in_luma_samples: r.read_ue("pic_height_in_luma_samples")?,
            conformance_window: Window::read(&mut r)?,
            bit_depth_luma_minus8: r.read_ue("bit_depth_luma_minus8")?,
            bit_depth_chroma_minus8: r.read_ue("bit_depth_chroma_minus8")?,
            log2_max_pic_order_cnt_lsb_minus4: r.read_ue("log2_max_pic_order_cnt_lsb_minus4")?,
            sub_layering_ordering_info: LayerInfo::read(&mut r, sps_max_sub_layers_minus1)?,
            log2_min_luma_coding_block_size_minus3: r
                .read_ue("log2_min_luma_coding_block_size_minus3")?,
            log2_diff_max_min_luma_coding_block_size: r
                .read_ue("log2_diff_max_min_luma_coding_block_size")?,
            log2_min_luma_transform_block_size_minus2: r
                .read_ue("log2_min_luma_transform_block_size_minus2")?,
            log2_diff_max_min_luma_transform_block_size: r
                .read_ue("log2_diff_max_min_luma_transform_block_size")?,
            max_transform_hierarchy_depth_inter: r
                .read_ue("max_transform_hierarchy_depth_inter")?,
            max_transform_hierarchy_depth_intra: r
                .read_ue("max_transform_hierarchy_depth_intra")?,
            scaling_list: ScalingList::read(&mut r)?,
            amp_enabled: r.read_bool("amp_enabled")?,
            sample_adaptive_offset_enabled: r.read_bool("sample_adaptive_offset_enabled")?,
            pcm: Pcm::read(&mut r)?,
            st_ref_pic_sets: ShortTermRefPicSet::read_with_count(&mut r)?,
            long_term_ref_pics_sps: LongTermRefPicSps::read(&mut r)?,
            sps_termporal_mvp_enabled: r.read_bool("sps_termporal_mvp_enabled")?,
            strong_intra_smoothing_enabled: r.read_bool("strong_intra_smoothing_enabled")?,
            vui_parameters: VuiParameters::read(&mut r, true, sps_max_sub_layers_minus1)?,
            sps_extension: SpsExtension::read(&mut r)?,
        };
        r.finish_rbsp()?;
        Ok(sps)
    }

    pub fn id(&self) -> SeqParamSetId {
        self.sps_seq_parameter_set_id
    }

    pub fn general_level(&self) -> Level {
        Level::from_level_idc(self.profile_tier_level.general_level_idc)
    }

    pub fn general_layer_profile(&self) -> &LayerProfile {
        self.profile_tier_level
            .general_profile
            .as_ref()
            .expect("SPS always has general profile")
    }

    pub fn general_tier(&self) -> Tier {
        self.general_layer_profile().tier()
    }

    /// Return the "lowest" compatible profile. A stream may conform to multiple profiles.
    pub fn general_profile(&self) -> Profile {
        self.general_layer_profile().profile()
    }

    /*
    fn read_log2_max_frame_num_minus4<R: BitRead>(r: &mut R) -> Result<u8, SpsError> {
        let val = r.read_ue("log2_max_frame_num_minus4")?;
        if val > 12 {
            Err(SpsError::Log2MaxFrameNumMinus4OutOfRange(val))
        } else {
            Ok(val as u8)
        }
    }

    /// returned value will be in the range 4 to 16 inclusive
    pub fn log2_max_frame_num(&self) -> u8 {
        self.log2_max_frame_num_minus4 + 4
    }

    /// Helper to calculate the pixel-dimensions of the video image specified by this SPS, taking
    /// into account sample-format, interlacing and cropping.
    pub fn pixel_dimensions(&self) -> Result<(u32, u32), SpsError> {
        let width = self
            .pic_width_in_mbs_minus1
            .checked_add(1)
            .and_then(|w| w.checked_mul(16))
            .ok_or_else(|| SpsError::FieldValueTooLarge {
                name: "pic_width_in_mbs_minus1",
                value: self.pic_width_in_mbs_minus1,
            })?;
        let mul = match self.frame_mbs_flags {
            FrameMbsFlags::Fields { .. } => 2,
            FrameMbsFlags::Frames => 1,
        };
        let vsub = if self.chroma_info.chroma_format == ChromaFormat::YUV420 {
            1
        } else {
            0
        };
        let hsub = if self.chroma_info.chroma_format == ChromaFormat::YUV420
            || self.chroma_info.chroma_format == ChromaFormat::YUV422
        {
            1
        } else {
            0
        };

        let step_x = 1 << hsub;
        let step_y = mul << vsub;

        let height = (self.pic_height_in_map_units_minus1 + 1)
            .checked_mul(mul * 16)
            .ok_or_else(|| SpsError::FieldValueTooLarge {
                name: "pic_height_in_map_units_minus1",
                value: self.pic_height_in_map_units_minus1,
            })?;
        if let Some(ref crop) = self.frame_cropping {
            let left_offset = crop.left_offset.checked_mul(step_x).ok_or_else(|| {
                SpsError::FieldValueTooLarge {
                    name: "left_offset",
                    value: crop.left_offset,
                }
            })?;
            let right_offset = crop.right_offset.checked_mul(step_x).ok_or_else(|| {
                SpsError::FieldValueTooLarge {
                    name: "right_offset",
                    value: crop.right_offset,
                }
            })?;
            let top_offset = crop.top_offset.checked_mul(step_y).ok_or_else(|| {
                SpsError::FieldValueTooLarge {
                    name: "top_offset",
                    value: crop.top_offset,
                }
            })?;
            let bottom_offset = crop.bottom_offset.checked_mul(step_y).ok_or_else(|| {
                SpsError::FieldValueTooLarge {
                    name: "bottom_offset",
                    value: crop.bottom_offset,
                }
            })?;
            let width = width
                .checked_sub(left_offset)
                .and_then(|w| w.checked_sub(right_offset));
            let height = height
                .checked_sub(top_offset)
                .and_then(|w| w.checked_sub(bottom_offset));
            if let (Some(width), Some(height)) = (width, height) {
                Ok((width, height))
            } else {
                Err(SpsError::CroppingError(crop.clone()))
            }
        } else {
            Ok((width, height))
        }
    }

    pub fn rfc6381(&self) -> rfc6381_codec::Codec {
        rfc6381_codec::Codec::avc1(self.profile_idc.0, self.constraint_flags.0, self.level_idc)
    }

    pub fn fps(&self) -> Option<f64> {
        let Some(vui) = &self.vui_parameters else {
            return None;
        };
        let Some(timing_info) = &vui.timing_info else {
            return None;
        };

        Some((timing_info.time_scale as f64) / (2.0 * (timing_info.num_units_in_tick as f64)))
    }
    */

    fn validate_max_num_sub_layers_minus1(max_num_sub_layers_minus1: u8) -> Result<(), SpsError> {
        if max_num_sub_layers_minus1 > 7 {
            Err(SpsError::FieldValueTooLarge {
                name: "max_num_sub_layers_minus1",
                value: max_num_sub_layers_minus1.into(),
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::rbsp::{self, decode_nal, BitReader};

    use super::*;
    use hex_literal::*;
    use test_case::test_case;

    #[test]
    fn test_it() {
        let data = hex!(
            "64 00 0A AC 72 84 44 26 84 00 00
            00 04 00 00 00 CA 3C 48 96 11 80"
        );
        let sps = SeqParameterSet::from_bits(rbsp::BitReader::new(&data[..])).unwrap();
        assert!(!format!("{:?}", sps).is_empty());
        assert_eq!(100, sps.profile_idc.0);
        assert_eq!(0, sps.constraint_flags.reserved_zero_two_bits());
        assert_eq!((64, 64), sps.pixel_dimensions().unwrap());
        assert!(!sps.rfc6381().to_string().is_empty())
    }

    #[test]
    fn test_dahua() {
        // From a Dahua IPC-HDW5231R-Z's sub stream, which is anamorphic.
        let data = hex!(
            "64 00 16 AC 1B 1A 80 B0 3D FF FF
           00 28 00 21 6E 0C 0C 0C 80 00 01
           F4 00 00 27 10 74 30 07 D0 00 07
           A1 25 DE 5C 68 60 0F A0 00 0F 42
           4B BC B8 50"
        );
        let sps = SeqParameterSet::from_bits(rbsp::BitReader::new(&data[..])).unwrap();
        println!("sps: {:#?}", sps);
        assert_eq!(
            sps.vui_parameters.unwrap().aspect_ratio_info.unwrap().get(),
            Some((40, 33))
        );
    }

    #[test]
    fn crop_removes_all_pixels() {
        let sps = SeqParameterSet {
            profile_idc: ProfileIdc(0),
            constraint_flags: ConstraintFlags(0),
            level_idc: 0,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo {
                chroma_format: ChromaFormat::Monochrome,
                separate_colour_plane_flag: false,
                bit_depth_luma_minus8: 0,
                bit_depth_chroma_minus8: 0,
                qpprime_y_zero_transform_bypass_flag: false,
                scaling_matrix: Default::default(),
            },
            log2_max_frame_num_minus4: 0,
            pic_order_cnt: PicOrderCntType::TypeTwo,
            max_num_ref_frames: 0,
            frame_cropping: Some(FrameCropping {
                bottom_offset: 20,
                left_offset: 20,
                right_offset: 20,
                top_offset: 20,
            }),
            pic_width_in_mbs_minus1: 1,
            pic_height_in_map_units_minus1: 1,
            frame_mbs_flags: FrameMbsFlags::Frames,
            gaps_in_frame_num_value_allowed_flag: false,
            direct_8x8_inference_flag: false,
            vui_parameters: None,
        };
        // should return Err, rather than assert due to integer underflow for example,
        let dim = sps.pixel_dimensions();
        assert!(matches!(dim, Err(SpsError::CroppingError(_))));
    }

    #[test_case(
        vec![
            0x67, 0x64, 0x00, 0x0c, 0xac, 0x3b, 0x50, 0xb0,
            0x4b, 0x42, 0x00, 0x00, 0x03, 0x00, 0x02, 0x00,
            0x00, 0x03, 0x00, 0x3d, 0x08,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(100),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 12,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 6,
            pic_order_cnt: PicOrderCntType::TypeTwo,
            max_num_ref_frames: 1,
            gaps_in_frame_num_value_allowed_flag: true,
            pic_width_in_mbs_minus1: 21,
            pic_height_in_map_units_minus1: 17,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: None,
            vui_parameters: Some(VuiParameters{
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1,
                    time_scale: 30,
                    fixed_frame_rate_flag: true,
                }),
                ..VuiParameters::default()
            }),
        },
        352,
        288,
        15.0; "352x288"
    )]
    #[test_case(
        vec![
            0x67, 0x64, 0x00, 0x1f, 0xac, 0xd9, 0x40, 0x50,
            0x05, 0xbb, 0x01, 0x6c, 0x80, 0x00, 0x00, 0x03,
            0x00, 0x80, 0x00, 0x00, 0x1e, 0x07, 0x8c, 0x18,
            0xcb,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(100),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 31,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 0,
            pic_order_cnt: PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4: 2
            },
            max_num_ref_frames: 4,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 79,
            pic_height_in_map_units_minus1: 44,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: None,
            vui_parameters: Some(VuiParameters{
                aspect_ratio_info: Some(AspectRatioInfo::Ratio1_1),
                video_signal_type: Some(VideoSignalType{
                    video_format: VideoFormat::Unspecified,
                    video_full_range_flag: true,
                    colour_description: None,
                }),
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1,
                    time_scale: 60,
                    fixed_frame_rate_flag: false,
                }),
                bitstream_restrictions: Some(BitstreamRestrictions{
                    motion_vectors_over_pic_boundaries_flag: true,
                    log2_max_mv_length_horizontal: 11,
                    log2_max_mv_length_vertical: 11,
                    max_num_reorder_frames: 2,
                    max_dec_frame_buffering: 4,
                    ..BitstreamRestrictions::default()
                }),
                ..VuiParameters::default()
            }),
        },
        1280,
        720,
        30.0; "1280x720"
    )]
    #[test_case(
        vec![
            0x67, 0x42, 0xc0, 0x28, 0xd9, 0x00, 0x78, 0x02,
            0x27, 0xe5, 0x84, 0x00, 0x00, 0x03, 0x00, 0x04,
            0x00, 0x00, 0x03, 0x00, 0xf0, 0x3c, 0x60, 0xc9, 0x20,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(66),
            constraint_flags: ConstraintFlags::from(0b11000000),
            level_idc: 40,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 0,
            pic_order_cnt: PicOrderCntType::TypeTwo,
            max_num_ref_frames: 3,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 119,
            pic_height_in_map_units_minus1: 67,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: Some(FrameCropping{
                bottom_offset: 4,
                ..FrameCropping::default()
            }),
            vui_parameters: Some(VuiParameters{
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1,
                    time_scale:      60,
                    fixed_frame_rate_flag: false,
                }),
                bitstream_restrictions: Some(BitstreamRestrictions{
                    motion_vectors_over_pic_boundaries_flag: true,
                    log2_max_mv_length_horizontal: 11,
                    log2_max_mv_length_vertical: 11,
                    max_dec_frame_buffering: 3,
                    ..BitstreamRestrictions::default()
                }),
                ..VuiParameters::default()
            }),
        },
        1920,
        1080,
        30.0; "1920x1080 baseline"
    )]
    #[test_case(
        vec![
            0x67, 0x64, 0x00, 0x28, 0xac, 0xd9, 0x40, 0x78,
            0x02, 0x27, 0xe5, 0x84, 0x00, 0x00, 0x03, 0x00,
            0x04, 0x00, 0x00, 0x03, 0x00, 0xf0, 0x3c, 0x60,
            0xc6, 0x58,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(100),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 40,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 0,
            pic_order_cnt: PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4: 2
            },
            max_num_ref_frames: 4,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 119,
            pic_height_in_map_units_minus1: 67,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: Some(FrameCropping{
                bottom_offset: 4,
                ..FrameCropping::default()
            }),
            vui_parameters: Some(VuiParameters{
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1,
                    time_scale: 60,
                    fixed_frame_rate_flag: false,
                }),
                bitstream_restrictions: Some(BitstreamRestrictions{
                    motion_vectors_over_pic_boundaries_flag: true,
                    log2_max_mv_length_horizontal: 11,
                    log2_max_mv_length_vertical: 11,
                    max_num_reorder_frames: 2,
                    max_dec_frame_buffering: 4,
                    ..BitstreamRestrictions::default()
                }),
                ..VuiParameters::default()
            }),
        },
        1920,
        1080,
        30.0; "1920x1080 nvidia"
    )]
    // This fails.

    /*#[test_case(
        vec![
            0x67, 0x64, 0x00, 0x29, 0xac, 0x13, 0x31, 0x40,
            0x78, 0x04, 0x47, 0xde, 0x03, 0xea, 0x02, 0x02,
            0x03, 0xe0, 0x00, 0x00, 0x03, 0x00, 0x20, 0x00,
            0x00, 0x06, 0x52, // 0x80,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(100),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 41,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 8,
            pic_order_cnt: PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4: 5
            },
            max_num_ref_frames: 4,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 119,
            pic_height_in_map_units_minus1: 33,
            frame_mbs_flags: FrameMbsFlags::Fields{
                mb_adaptive_frame_field_flag: false,
            },
            direct_8x8_inference_flag: true,
            frame_cropping: Some(FrameCropping{
                bottom_offset: 2,
                ..FrameCropping::default()
            }),
            vui_parameters: Some(VuiParameters{
                aspect_ratio_info: Some(AspectRatioInfo::Ratio1_1),
                overscan_appropriate: OverscanAppropriate::Appropriate,
                video_signal_type: Some(VideoSignalType{
                    video_format: VideoFormat::Unspecified,
                    video_full_range_flag: false,
                    colour_description: Some(ColourDescription{
                        colour_primaries: 1,
                        transfer_characteristics: 1,
                        matrix_coefficients: 1,
                    }),
                }),
                chroma_loc_info: Some(ChromaLocInfo{
                    chroma_sample_loc_type_top_field: 0,
                    chroma_sample_loc_type_bottom_field: 0,
                }),
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1,
                    time_scale: 50,
                    fixed_frame_rate_flag: true,
                }),
                pic_struct_present_flag: true,
                ..VuiParameters::default()
            }),
        },
        1920,
        1084,
        25.0; "1920x1080"
    )]
    */
    #[test_case(
        vec![103, 100, 0, 32, 172, 23, 42, 1, 64, 30, 104, 64, 0, 1, 194, 0, 0, 87, 228, 33],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(100),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 32,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 10,
            pic_order_cnt: PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4: 4
            },
            max_num_ref_frames: 1,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 79,
            pic_height_in_map_units_minus1: 59,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: None,
            vui_parameters: Some(VuiParameters{
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1800,
                    time_scale: 90000,
                    fixed_frame_rate_flag: true,
                }),
                ..VuiParameters::default()
            }),
        },
        1280,
        960,
        25.0; "hikvision"
    )]
    #[test_case(
        vec![
            103, 100, 0, 50, 173, 132, 1, 12, 32, 8, 97, 0, 67, 8, 2,
            24, 64, 16, 194, 0, 132, 59, 80, 20, 0, 90, 211,
            112, 16, 16, 20, 0, 0, 3, 0, 4, 0, 0, 3, 0, 162, 16,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(100),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 50,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            /*seq_scaling_list: Some(SeqScalingList{
                scaling_list_4x4: vec![
                    vec![
                        16, 16, 16, 16, 16, 16, 16, 16,
                        16, 16, 16, 16, 16, 16, 16, 16,
                    ],
                    vec![
                        16, 16, 16, 16, 16, 16, 16, 16,
                        16, 16, 16, 16, 16, 16, 16, 16,
                    ],
                    vec![
                        16, 16, 16, 16, 16, 16, 16, 16,
                        16, 16, 16, 16, 16, 16, 16, 16,
                    ],
                    vec![
                        16, 16, 16, 16, 16, 16, 16, 16,
                        16, 16, 16, 16, 16, 16, 16, 16,
                    ],
                    vec![
                        16, 16, 16, 16, 16, 16, 16, 16,
                        16, 16, 16, 16, 16, 16, 16, 16,
                    ],
                    vec![
                        16, 16, 16, 16, 16, 16, 16, 16,
                        16, 16, 16, 16, 16, 16, 16, 16,
                    ],
                ],
                use_default_scaling_matrix_4x4_flag: vec![false, false, false, false, false, false],
                ..SeqScalingList::default()
            }),*/
            log2_max_frame_num_minus4: 6,
            pic_order_cnt: PicOrderCntType::TypeTwo,
            max_num_ref_frames: 1,
            gaps_in_frame_num_value_allowed_flag: true,
            pic_width_in_mbs_minus1: 159,
            pic_height_in_map_units_minus1: 89,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: None,
            vui_parameters: Some(VuiParameters{
                video_signal_type: Some(VideoSignalType{
                    video_format: VideoFormat::Unspecified,
                    video_full_range_flag: true,
                    colour_description: Some(ColourDescription{
                        colour_primaries: 1,
                        transfer_characteristics: 1,
                        matrix_coefficients: 1,
                    }),
                }),
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1,
                    time_scale: 40,
                    fixed_frame_rate_flag: true,
                }),
                ..VuiParameters::default()
            }),
        },
        2560,
        1440,
        20.0; "scaling matrix"
    )]
    #[test_case(
        vec![
            103, 100, 0, 42, 172, 44, 172, 7,
            128, 34, 126, 92, 5, 168, 8, 8,
            10, 0, 0, 7, 208, 0, 3, 169,
            129, 192, 0, 0, 76, 75, 0, 0,
            38, 37, 173, 222, 92, 20,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(100),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 42,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 4,
            pic_order_cnt: PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4: 4
            },
            max_num_ref_frames: 2,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 119,
            pic_height_in_map_units_minus1: 67,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: Some(FrameCropping{
                bottom_offset: 4,
                ..FrameCropping::default()
            }),
            vui_parameters: Some(VuiParameters{
                aspect_ratio_info: Some(AspectRatioInfo::Ratio1_1),
                video_signal_type: Some(VideoSignalType{
                    video_format: VideoFormat::Unspecified,
                    video_full_range_flag: false,
                    colour_description: Some(ColourDescription{
                        colour_primaries: 1,
                        transfer_characteristics: 1,
                        matrix_coefficients: 1,
                    }),
                }),
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1000,
                    time_scale: 120000,
                    fixed_frame_rate_flag: true,
                }),
                nal_hrd_parameters: Some(HrdParameters{
                    cpb_specs: vec![CpbSpec{
                        bit_rate_value_minus1: 39061,
                        cpb_size_value_minus1: 156249,
                        cbr_flag: true,
                    }],
                    initial_cpb_removal_delay_length_minus1: 23,
                    cpb_removal_delay_length_minus1: 15,
                    dpb_output_delay_length_minus1: 5,
                    time_offset_length: 24,
                    ..HrdParameters::default()
                }),
                low_delay_hrd_flag: Some(false),
                pic_struct_present_flag: true,
                ..VuiParameters::default()
            }),
        },
        1920,
        1080,
        60.0; "1920x1080 nvenc hrd"
    )]
    #[test_case(
        vec![
            103, 77, 0, 41, 154, 100, 3, 192,
            17, 63, 46, 2, 220, 4, 4, 5,
            0, 0, 3, 3, 232, 0, 0, 195,
            80, 232, 96, 0, 186, 180, 0, 2,
            234, 196, 187, 203, 141, 12, 0, 23,
            86, 128, 0, 93, 88, 151, 121, 112,
            160,
        ],
        SeqParameterSet{
            profile_idc: ProfileIdc::from(77),
            constraint_flags: ConstraintFlags::from(0),
            level_idc: 41,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo{
                chroma_format: ChromaFormat::YUV420,
                ..ChromaInfo::default()
            },
            log2_max_frame_num_minus4: 5,
            pic_order_cnt: PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4: 5
            },
            max_num_ref_frames: 1,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 119,
            pic_height_in_map_units_minus1: 67,
            frame_mbs_flags: FrameMbsFlags::Frames,
            direct_8x8_inference_flag: true,
            frame_cropping: Some(FrameCropping{
                bottom_offset: 4,
                ..FrameCropping::default()
            }),
            vui_parameters: Some(VuiParameters{
                aspect_ratio_info: Some(AspectRatioInfo::Ratio1_1),
                video_signal_type: Some(VideoSignalType{
                    video_format: VideoFormat::Unspecified,
                    video_full_range_flag: true,
                    colour_description: Some(ColourDescription{
                        colour_primaries: 1,
                        transfer_characteristics: 1,
                        matrix_coefficients: 1,
                    }),
                }),
                timing_info: Some(TimingInfo{
                    num_units_in_tick: 1000,
                    time_scale: 50000,
                    fixed_frame_rate_flag: true,
                }),
                nal_hrd_parameters: Some(HrdParameters{
                    bit_rate_scale: 4,
                    cpb_size_scale: 3,
                    cpb_specs: vec![CpbSpec{
                        bit_rate_value_minus1: 11948,
                        cpb_size_value_minus1: 95585,
                        cbr_flag: false,
                    }],
                    initial_cpb_removal_delay_length_minus1: 23,
                    cpb_removal_delay_length_minus1: 15,
                    dpb_output_delay_length_minus1: 5,
                    time_offset_length: 24,
                }),
                vcl_hrd_parameters: Some(HrdParameters{
                    bit_rate_scale: 4,
                    cpb_size_scale: 3,
                    cpb_specs: vec![CpbSpec{
                        bit_rate_value_minus1: 11948,
                        cpb_size_value_minus1: 95585,
                        cbr_flag: false,
                    }],
                    initial_cpb_removal_delay_length_minus1: 23,
                    cpb_removal_delay_length_minus1: 15,
                    dpb_output_delay_length_minus1: 5,
                    time_offset_length: 24,
                    ..HrdParameters::default()
                }),
                low_delay_hrd_flag: Some(false),
                pic_struct_present_flag: true,
                ..VuiParameters::default()
            }),
        },
        1920,
        1080,
        25.0; "1920x1080 hikvision nal hrd + vcl hrd"
    )]
    fn test_sps(byts: Vec<u8>, sps: SeqParameterSet, width: u32, height: u32, fps: f64) {
        let sps_rbsp = decode_nal(&byts).unwrap();
        let sps2 = SeqParameterSet::from_bits(BitReader::new(&*sps_rbsp)).unwrap();

        let (width2, height2) = sps2.pixel_dimensions().unwrap();
        assert_eq!(sps, sps2);
        assert_eq!(width, width2);
        assert_eq!(height, height2);
        assert_eq!(fps, sps2.fps().unwrap());
    }
}
