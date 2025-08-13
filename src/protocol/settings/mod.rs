//! Settings protocol definitions for the high flow NEXT device.
//!
//! This module contains all types and decoding logic related to device
//! configuration and runtime settings. The `Settings` struct is the top-
//! level container.

mod alarm;
mod display;
mod lighting;
mod sensor;
mod system;

use std::{array::from_fn, ops::BitAnd};

use crate::{
    define_wrapped, impl_ranged,
    misc::{Decode, Guard, GuardOutput, IoError, Reader},
};

pub use self::alarm::*;
pub use self::display::*;
pub use self::lighting::*;
pub use self::sensor::*;
pub use self::system::*;

/// Settings of a high flow NEXT device
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Settings {
    /// System related settings.
    pub system: SystemSettings,

    /// Sensor related settings.
    pub sensor: SensorSettings,

    /// Alarm related settings.
    pub alarms: AlarmSettings,

    /// Display related settings.
    pub display: DisplaySettings,

    /// Lighting / `RGBpx` related settings.
    pub lighting: Option<LightingSettings>,
}

impl Decode for Settings {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let _version = reader.read_u16be()?;

        let display = DisplaySettings::decode(reader)?;

        let increased_current_draw = Option::<CurrentDraw>::decode(reader)?;
        let aqua_bus_address = AquaBusAddress::decode(reader)?;

        let water_temp_offset = TempOffset::decode(reader)?;
        let exrernal_temp_offset = TempOffset::decode(reader)?;
        let medium = Medium::decode(reader)?;
        let connector_type = ConnectorType::decode(reader)?;

        let flow_correction_values = <[FlowCorrection; 10]>::decode(reader)?;
        let flow_correction_flows = <[Flow; 10]>::decode(reader)?;
        let flow_correction = R::guard(|x| {
            let flow_correction_flows = x.extract(flow_correction_flows);
            let flow_correction_values = x.extract(flow_correction_values);

            from_fn(|i| (flow_correction_flows[i], flow_correction_values[i]))
        });

        let lighting = Option::<LightingSettings>::decode(reader)?;

        let standby_flags = StandbyFlags::decode(reader)?;
        reader.skip::<2>()?;
        let conductivity_offset = ConductivityOffset::decode(reader)?;
        let water_quality_max = Conductivity::decode(reader)?;
        let water_quality_min = Conductivity::decode(reader)?;
        reader.skip::<1>()?;
        let power_flags = PowerFlags::decode(reader)?;
        let power_damping = PowerDamping::decode(reader)?;
        let alarms = AlarmSettings::decode(reader)?;
        reader.skip::<1>()?;

        Ok(R::guard(|x| {
            let system = SystemSettings {
                standby_flags: x.extract(standby_flags),
                aqua_bus_address: x.extract(aqua_bus_address),
                increased_current_draw: x.extract(increased_current_draw),
            };

            let sensor = SensorSettings {
                medium: x.extract(medium),
                connector_type: x.extract(connector_type),
                flow_correction: x.extract(flow_correction),
                water_temp_offset: x.extract(water_temp_offset),
                external_temp_offset: x.extract(exrernal_temp_offset),
                conductivity_offset: x.extract(conductivity_offset),
                water_quality_max: x.extract(water_quality_max),
                water_quality_min: x.extract(water_quality_min),
                power_flags: x.extract(power_flags),
                power_damping: x.extract(power_damping),
            };

            Self {
                system,
                sensor,
                alarms: x.extract(alarms),
                display: x.extract(display),
                lighting: x.extract(lighting),
            }
        }))
    }
}

define_wrapped! {
    /// Water flow value.
    ///
    /// Valid value range: 0..3000 in 1/10 liter / gallons per second (1/10 l/sec / gal/sec)
    pub type Flow<u16, FlowTag>;
}
impl_ranged!(Flow<u16, FlowTag>, 0, 3000);

#[inline]
fn flag_set<T>(flags: T, flag: T) -> bool
where
    T: BitAnd<Output = T> + Default + Eq,
{
    flags & flag != T::default()
}
