use bitflags::bitflags;

use crate::misc::{Decode, GuardOutput, IoError, Reader};
use crate::{define_wrapped, impl_ranged};

use super::Flow;

/// Sensor related settings for a high flow NEXT device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SensorSettings {
    /// Medium that is used as coolant.
    pub medium: Medium,

    /// Connector type that is used to connect the device to the piping system.
    pub connector_type: ConnectorType,

    /// Flow correction values (as pair of [`Flow`] to [`FlowCorrection`]).
    pub flow_correction: [(Flow, FlowCorrection); 10],

    /// Water temperature offset (to adjust the sensor).
    pub water_temp_offset: TempOffset,

    /// External temperature offset (to adjust the sensor).
    pub external_temp_offset: TempOffset,

    /// Conductivity offset (to adjust the sensor).
    pub conductivity_offset: ConductivityOffset,

    /// Conductivity that maps to the maximum water quality (100%).
    pub water_quality_max: Conductivity,

    /// Conductivity that maps to the minimum water quality (0%).
    pub water_quality_min: Conductivity,

    /// Flags for the power calculation.
    pub power_flags: PowerFlags,

    /// Power damping
    pub power_damping: PowerDamping,
}

/// Medium that is used as coolant.
///
/// Used in [`SensorSettings::medium`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Medium {
    /// DP Ultra
    DpUltra,

    /// Distilled water.
    DistilledWater,
}

impl Decode for Medium {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Self::DpUltra)),
            0x01 => Ok(R::guard(|_| Self::DistilledWater)),
            x => Err(IoError::InvalidValue("Medium", x.into())),
        }
    }
}

/// Connector type that is used to connect the device to the piping system.
///
/// Used in [`SensorSettings::connector_type`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConnectorType {
    /// Inner diameter > 7mm
    InnerDiameterGt7mm,

    /// Inner diameter < 7mm
    InnerDiameterLt7mm,
}

impl Decode for ConnectorType {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Self::InnerDiameterGt7mm)),
            0x01 => Ok(R::guard(|_| Self::InnerDiameterLt7mm)),
            x => Err(IoError::InvalidValue("ConnectorType", x.into())),
        }
    }
}

bitflags! {
    /// Flags to control the power calculation.
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct PowerFlags: u8 {
        /// Enable automatic offset compensation in standby.
        const AUTOMATIC_POWER_OFFSET_COMPENSATION = 0x01;
    }
}

impl Decode for PowerFlags {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let bits = reader.read_u8()?;

        Ok(R::guard(|_| Self::from_bits_truncate(bits)))
    }
}

define_wrapped! {
    /// Flow correction value used in [`SensorSettings::flow_correction`].
    ///
    /// Valid value range: -5000..5000 in 1/100 percent (1/100 %)
    pub type FlowCorrection<i16, FlowCorrectionTag>;
}
impl_ranged!(FlowCorrection<i16, FlowCorrectionTag>, -5000, 5000);

define_wrapped! {
    /// Water temperature offset used in [`SensorSettings`].
    ///
    /// Valid value range: -1500..1500 in 1/100 degree celsius / fahrenheit (1/100 °C / °F)
    pub type TempOffset<i16, TempOffsetTag>;
}
impl_ranged!(TempOffset<i16, TempOffsetTag>, -1500, 1500);

define_wrapped! {
    /// Power damping used in [`SensorSettings::power_damping`].
    ///
    /// Valid value range: 0..10000 in milli watts (1/1000 W)
    pub type PowerDamping<u16, PowerDampingTag>;
}
impl_ranged!(PowerDamping<u16, PowerDampingTag>, 0, 10_000);

define_wrapped! {
    /// Conductivity to define the water quality used in [`SensorSettings`].
    ///
    /// Valid value range: 0..2000 in micro siemens per centimeter (µS/cm)
    pub type Conductivity<u16, ConductivityTag>;
}
impl_ranged!(Conductivity<u16, ConductivityTag>, 0, 2000);

define_wrapped! {
    /// Conductivity offset (to adjust the sensor) used in [`SensorSettings::conductivity_offset`]
    ///
    /// Valid value range: -500..500 in 1/10 micro siemens per centimeter (1/10 µS/cm)
    pub type ConductivityOffset<i16, ConductivityOffsetTag>;
}
impl_ranged!(ConductivityOffset<i16, ConductivityOffsetTag>, -500, 500 );
