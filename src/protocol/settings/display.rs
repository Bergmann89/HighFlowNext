use bitflags::bitflags;

use crate::misc::{Decode, Guard, GuardOutput, IoError, Reader};
use crate::{define_wrapped, impl_ranged};

/// Display related settings for a high flow NEXT device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DisplaySettings {
    /// Unit do display temperatures in.
    pub temperature_unit: TemperatureUnit,

    /// Unit to display the flow in.
    pub flow_unit: FlowUnit,

    /// Different flags.
    pub display_flags: DisplayFlags,

    /// Interval to cycle through the different pages.
    pub next_page_interval: Option<NextPageInterval>,

    /// Pages to show on the display.
    pub page_flags: PageFlags,

    /// Display brightness (during normal operation).
    pub display_brightness: DisplayBrightness,

    /// Display brightness (during stand by).
    pub idle_display_brightness: Option<DisplayBrightness>,

    /// Settings of the different value charts.
    pub charts: [Chart; 4],
}

impl Decode for DisplaySettings {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let temperature_unit = TemperatureUnit::decode(reader)?;
        let flow_unit = FlowUnit::decode(reader)?;
        reader.skip::<1>()?;
        let next_page_interval = Option::<NextPageInterval>::decode(reader)?;
        reader.skip::<2>()?;
        let page_flags = PageFlags::decode(reader)?;
        reader.skip::<4>()?;
        let display_brightness = DisplayBrightness::decode(reader)?;
        let idle_display_brightness = Option::<DisplayBrightness>::decode(reader)?;
        reader.skip::<4>()?;
        let display_flags = DisplayFlags::decode(reader)?;
        let charts = <[Chart; 4] as Decode>::decode(reader)?;

        Ok(R::guard(|x| Self {
            temperature_unit: x.extract(temperature_unit),
            flow_unit: x.extract(flow_unit),
            display_flags: x.extract(display_flags),
            next_page_interval: x.extract(next_page_interval),
            page_flags: x.extract(page_flags),
            display_brightness: x.extract(display_brightness),
            idle_display_brightness: x.extract(idle_display_brightness),
            charts: x.extract(charts),
        }))
    }
}

/// Settings for a value chart displayed on the display of the device.
///
/// Used in [`DisplaySettings::charts`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Chart {
    /// Source of the data that is displayed in the chart.
    pub source: ChartSource,

    /// Interval the chart should be updated in.
    pub interval: ChartInterval,
}

impl Decode for Chart {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        reader.skip::<1>()?;
        let source = ChartSource::decode(reader)?;
        let interval = ChartInterval::decode(reader)?;

        Ok(R::guard(|x| Self {
            source: x.extract(source),
            interval: x.extract(interval),
        }))
    }
}

/// Unit do display temperatures in.
///
/// Used in [`DisplaySettings::temperature_unit`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TemperatureUnit {
    /// Degree Celsius (°C)
    C,

    /// Degree Fahrenheit (°F)
    F,
}

impl Decode for TemperatureUnit {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Self::C)),
            0x01 => Ok(R::guard(|_| Self::F)),
            x => Err(IoError::InvalidValue("TemperatureUnit", x.into())),
        }
    }
}

/// Unit do display the flow in.
///
/// Used in [`DisplaySettings::flow_unit`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FlowUnit {
    /// Liter per hour (L/h).
    Liter,

    /// Gallons per hour (gal/h)
    Gallons,
}

impl Decode for FlowUnit {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Self::Liter)),
            0x01 => Ok(R::guard(|_| Self::Gallons)),
            x => Err(IoError::InvalidValue("FlowUnit", x.into())),
        }
    }
}

/// Display brightness.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DisplayBrightness {
    /// Maximum display brightness.
    Maximum,

    /// Medium display brightness.
    Medium,

    /// Low display brightness.
    Low,
}

impl Decode for DisplayBrightness {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Self::Maximum)),
            0x01 => Ok(R::guard(|_| Self::Medium)),
            0x02 => Ok(R::guard(|_| Self::Low)),
            x => Err(IoError::InvalidValue("DisplayBrightness", x.into())),
        }
    }
}

impl Decode for Option<DisplayBrightness> {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Some(DisplayBrightness::Maximum))),
            0x01 => Ok(R::guard(|_| Some(DisplayBrightness::Medium))),
            0x02 => Ok(R::guard(|_| Some(DisplayBrightness::Low))),
            0x03 => Ok(R::guard(|_| None)),
            x => Err(IoError::InvalidValue("Option<DisplayBrightness>", x.into())),
        }
    }
}

/// Source of the data that is displayed in the chart.
///
/// Used in [`Chart::source`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ChartSource {
    /// Current water flow.
    Flow,

    /// Current water temperature.
    WaterTemp,

    /// Current external temperature.
    ExternalTemp,

    /// Current conductivity.
    Conductivity,

    /// Current water quality
    WaterQuality,

    /// Current power consumption.
    PowerConsumption,

    /// Current system voltage (5V)
    SystemVoltage,
}

impl Decode for ChartSource {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u8()? {
            0x00 => Ok(R::guard(|_| Self::Flow)),
            0x01 => Ok(R::guard(|_| Self::WaterTemp)),
            0x02 => Ok(R::guard(|_| Self::ExternalTemp)),
            0x03 => Ok(R::guard(|_| Self::Conductivity)),
            0x04 => Ok(R::guard(|_| Self::WaterQuality)),
            0x05 => Ok(R::guard(|_| Self::PowerConsumption)),
            0x06 => Ok(R::guard(|_| Self::SystemVoltage)),
            x => Err(IoError::InvalidValue("ChartSource", x.into())),
        }
    }
}

bitflags! {
    /// Different flags uses in [`DisplaySettings::flags`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct DisplayFlags: u8 {
        /// Rotate the display by 180°
        const ROTATE = 0x01;

        /// Invert the color of the display.
        const INVERT = 0x04;

        /// Automatically invert the color of the display.
        const AUTO_INVERT = 0x08;

        /// Disable the device buttons.
        const DISABLE_BUTTONS = 0x10;

        /// Lock the device menu.
        const LOCK_MENU = 0x20;
    }
}

impl Decode for DisplayFlags {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let bits = reader.read_u8()?;

        Ok(R::guard(|_| Self::from_bits_truncate(bits)))
    }
}

bitflags! {
    /// Different pages displayed on the device display.
    ///
    /// Uses in [`DisplaySettings::page_flags`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct PageFlags: u16 {
        /// Show the device logo page.
        const DEVICE_INFO = 0x0001;

        /// Show the current water flow page.
        const FLOW = 0x0002;

        /// Show the current water temperature page.
        const WATER_TEMP = 0x0004;

        /// Show the current external temperature page.
        const EXTERNAL_TEMP = 0x0008;

        /// Show the current conductivity page.
        const CONDUCTIVITY = 0x0010;

        /// Show the current water quality page.
        const WATER_QUALITY = 0x0020;

        /// Show the current volume counter page.
        const VOLUME_COUNT = 0x0040;

        /// Show the current power consumption page.
        const POWER_SENSOR = 0x0080;

        /// Show the flow / water temperature page.
        const FLOW_WATERTEMP = 0x0100;

        /// Show the conductivity / water quality page.
        const COND_QUALITY = 0x0200;

        /// Show the water temperature / external temperature page.
        const TEMPERATURES = 0x0400;

        /// Show the water flow / water volume page.
        const FLOW_VOLUME = 0x0800;

        /// Show chart #1.
        const CHART1 = 0x1000;

        /// Show chart #2.
        const CHART2 = 0x2000;

        /// Show chart #3.
        const CHART3 = 0x4000;

        /// Show chart #4.
        const CHART4 = 0x8000;
    }
}

impl Decode for PageFlags {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let bits = reader.read_u16be()?;

        Ok(R::guard(|_| Self::from_bits_truncate(bits)))
    }
}

define_wrapped! {
    /// Define the interval to update the different data charts in.
    ///
    /// Used in [`Chart::interval`].
    ///
    /// Valid value range: 1..60000 in 1/10 of a second
    pub type ChartInterval<u16, ChartIntervalTag>;
}
impl_ranged!(ChartInterval<u16, ChartIntervalTag>, 1, 60_000);

define_wrapped! {
    /// Define the interval to switch between the different pages.
    ///
    /// Used in [`DisplaySettings::next_page_interval`].
    ///
    /// Valid value range: 3..60 in seconds
    pub type NextPageInterval<u8, NextPageIntervalTag>;
}
impl_ranged!(NextPageInterval<u8, NextPageIntervalTag>, 3, 60);

impl Decode for Option<NextPageInterval> {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let val = reader.read_u8()?;

        let ret = R::guard(|_| {
            if val <= NextPageInterval::max_inclusive() {
                let val = NextPageInterval::from_value(val)
                    .map_err(|_| IoError::InvalidValue("NextPageInterval", val.into()))?;

                Ok(Some(val))
            } else {
                Ok(None)
            }
        });

        R::Guard::transpose_result(ret)
    }
}
