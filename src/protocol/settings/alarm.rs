use bitflags::bitflags;

use crate::{
    define_wrapped, impl_ranged,
    misc::{Decode, Guard, GuardOutput, IoError, Reader},
};

use super::{flag_set, Flow};

/// Alarm related settings for a high flow NEXT device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlarmSettings {
    /// Different flags.
    pub flags: AlarmFlags,

    /// Time to disable the alarms after boot to give the system time to start.
    pub startup_delay: StartupDelay,

    /// Whether to raise an alarm if the flow of the device drops below the
    /// configured value or not.
    pub flow_alarm_limit: Option<Flow>,

    /// Whether to raise an alarm if the water temperature of the device raises
    /// above the configured value or not.
    pub water_temperature_limit: Option<Temperature>,

    /// Whether to raise an alarm if the external temperature of the device raises
    /// above the configured value or not.
    pub external_temperature_limit: Option<Temperature>,

    /// Whether to raise an alarm if the water quality of the device drops below
    /// the configured value or not.
    pub water_quality_limit: Option<WaterQuality>,

    /// Signal to output at the signal output connector.
    pub output_signal: OutputSignal,
}

impl Decode for AlarmSettings {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let flags = AlarmFlags::decode(reader)?;
        let alarms = reader.read_u8()?;
        reader.skip::<1>()?;
        let startup_delay = StartupDelay::decode(reader)?;
        let flow_alarm_limit = Flow::decode_opt(reader, flag_set(alarms, 0x01))?;
        let water_temperature_limit = Temperature::decode_opt(reader, flag_set(alarms, 0x02))?;
        let external_temperature_limit = Temperature::decode_opt(reader, flag_set(alarms, 0x04))?;
        let water_quality_limit = WaterQuality::decode_opt(reader, flag_set(alarms, 0x08))?;
        let output_signal = OutputSignal::decode(reader)?;

        Ok(R::guard(|x| Self {
            flags: x.extract(flags),
            startup_delay: x.extract(startup_delay),
            flow_alarm_limit: x.extract(flow_alarm_limit),
            water_temperature_limit: x.extract(water_temperature_limit),
            external_temperature_limit: x.extract(external_temperature_limit),
            water_quality_limit: x.extract(water_quality_limit),
            output_signal: x.extract(output_signal),
        }))
    }
}

/// Signal to output at the signal output connector.
///
/// Used in [`AlarmSettings::output_signal`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputSignal {
    /// Generate a constant speed signal.
    ConstantSpeed,

    /// Generate high flow sensor (53069) signal.
    HighFlowSensor,

    /// Generate a fan speed signal from the flow rate (1000rpm = 100l/h).
    FanFromFlow,

    /// Power switch for 1 seconds at alarm.
    PulseOnAlarm,

    /// Permanently switch on the output signal.
    PermanentOn,

    /// Permanently switch off the output signal
    PermanentOff,
}

impl Decode for OutputSignal {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Self::ConstantSpeed)),
            0x01 => Ok(R::guard(|_| Self::HighFlowSensor)),
            0x02 => Ok(R::guard(|_| Self::FanFromFlow)),
            0x03 => Ok(R::guard(|_| Self::PulseOnAlarm)),
            0x04 => Ok(R::guard(|_| Self::PermanentOn)),
            0x05 => Ok(R::guard(|_| Self::PermanentOff)),
            x => Err(IoError::InvalidValue("OutputSignal", x.into())),
        }
    }
}

bitflags! {
    /// Different flags uses in [`AlarmSettings::flags`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct AlarmFlags: u8 {
        /// Disable signal output during alarm.
        const DISABLE_SIGNAL_OUTPUT_DURING_ALARM = 0x20;

        /// Enable optical indicator during alarm (red ring flashing).
        const ENABLE_OPTICAL_INDICATOR = 0x40;

        /// Enable acustic indicator during alarm (beep tone).
        const ENABLE_ACUSTIC_INDICATOR = 0x80;
    }
}

impl Decode for AlarmFlags {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let bits = reader.read_u8()?;

        Ok(R::guard(|_| Self::from_bits_truncate(bits)))
    }
}

define_wrapped! {
    /// Time to disable the alarms after boot to give the system time to start.
    ///
    /// Used in [`AlarmSettings::startup_delay`].
    ///
    /// Valid value range: 0..100 in seconds
    pub type StartupDelay<u8, StartupDelayTag>;
}
impl_ranged!(StartupDelay<u8, StartupDelayTag>, 0, 100);

define_wrapped! {
    /// Temperature value.
    ///
    /// Valid value range: 0..10000 in 1/100 degree celsius / fahrenheit (1/100 °C / °F)
    pub type Temperature<u16, TemperatureTag>;
}
impl_ranged!(Temperature<u16, TemperatureTag>, 0, 10_000);

define_wrapped! {
    /// Water quality value.
    ///
    /// Valid value range: 0..10000 in 1/100 percent (1/100 %)
    pub type WaterQuality<u16, WaterQualityTag>;
}
impl_ranged!(WaterQuality<u16, WaterQualityTag>, 0, 10000);
