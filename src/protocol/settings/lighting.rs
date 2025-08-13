use std::array::from_fn;

use arrayvec::ArrayVec;
use color_space::{FromRgb, Hsv, Rgb};

use crate::misc::{Decode, Guard, GuardOutput, IoError, Reader};
use crate::{define_wrapped, impl_ranged, impl_verify_simple};

use super::flag_set;

/// Lighting / `RGBpx` related settings for a high flow NEXT device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LightingSettings {
    /// General Brightness of all LED effects.
    pub brightness: Brightness,

    /// List of controllers for the LED strip (external connector)
    pub strip_controllers: ArrayVec<Controller, 6>,

    /// List of controllers for the
    pub sensor_controllers: ArrayVec<Controller, 2>,
}

impl Decode for Option<LightingSettings> {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let brightness = Brightness::decode(reader)?;
        reader.skip::<1>()?;
        let flags = reader.read_u8()?;
        reader.skip::<1>()?;

        if flags & 0x02 != 0 {
            <[Option<Controller>; 8]>::skip_bytes(reader)?;

            Ok(R::guard(|_| None))
        } else {
            let strip_controllers = <[Option<Controller>; 6]>::decode(reader)?;
            let sensor_controllers = <[Option<Controller>; 2]>::decode(reader)?;

            Ok(R::guard(|x| {
                let brightness = x.extract(brightness);
                let strip_controllers = x.extract(strip_controllers);
                let sensor_controllers = x.extract(sensor_controllers);

                Some(LightingSettings {
                    brightness,
                    strip_controllers: strip_controllers.into_iter().flatten().collect(),
                    sensor_controllers: sensor_controllers.into_iter().flatten().collect(),
                })
            }))
        }
    }
}

/// Defines a single LED effect.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Controller {
    /// Offset in the LED strip (in number of LEDs).
    pub offset: u8,

    /// Length of the effect (in number of LEDs).
    pub length: u8,

    /// Effect displayed for the specified region.
    pub effect: Effect,

    /// Source for data controlled effects.
    pub data_source: Option<DataSource>,

    /// Filtering of fluctuating rising values.
    pub sensor_attenuation_rising: u8,

    /// Filtering of fluctuating falling values.
    pub sensor_attenuation_falling: u8,
}

/// Defines different effects that are displayed for a specific [`Controller`].
#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    Static(EffectStatic),
    Breathing(EffectBreathing),
    Rainbow(EffectRainbow),
    Blink(EffectBlink),
    ColorChange(EffectColorChange),
    Sequence(EffectSequence),
    Scanner(EffectScanner),
    Laser(EffectScanner),
    Wave(EffectWave),
    ColorSequence(EffectColorSequence),
    ColorShift(EffectColorShift),
    BarGraph(EffectBarGraph),
    Flame(EffectFlame),
    Rain(EffectRain),
    Snow(EffectRain),
    Stardust(EffectRain),
    ColorSwitch(EffectColorSwitch),
    SwipingRainbow(EffectSwipingRainbow),
    SoundFlash(EffectSoundFlash),
    SoundBars(EffectBarGraph),
    SoundSlider(EffectSoundSlider),
    SoundShift(EffectSoundShift),
    Ambient(EffectAmbient),
    ColorGradient(EffectColorGradient),
}

/// A static RGB effect with a single constant color.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectStatic {
    /// The static display color.
    pub color: Color,

    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
    /// Optional external control for saturation.
    pub source_control_saturation: Option<SourceControl>,
}

/// A breathing effect that smoothly fades a color in and out.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectBreathing {
    /// The breathing base color.
    pub color: Color,

    /// Breathing speed (cycle frequency).
    pub speed: EffectPercent,
    /// Breathing intensity (amplitude of brightness change).
    pub intensity: EffectPercent,
    /// Delay at maximum brightness.
    pub delay_max_brightness: EffectDelay,
    /// Delay at minimum brightness.
    pub delay_min_brightness: EffectDelay,

    /// Optional external control for breathing speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for breathing intensity.
    pub source_control_intensity: Option<SourceControl>,
}

/// A rainbow effect cycling through a color spectrum.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectRainbow {
    /// Base color (used as reference).
    pub color: Color,

    /// Cycle speed of the rainbow.
    pub speed: EffectPercent,
    /// Range of colors included in the spectrum.
    pub color_range: EffectPercent,

    /// Reverse the direction of the color cycle.
    pub reverse_direction: bool,

    /// Optional external control for cycle speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A blinking effect alternating between background and foreground colors.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct EffectBlink {
    /// Background color while blinking.
    pub background: Color,
    /// Foreground blink colors.
    pub colors: ArrayVec<Color, 5>,

    /// Blinking speed.
    pub speed: EffectPercent,

    /// Fade in foreground colors.
    pub fade_in: bool,
    /// Fade out foreground colors.
    pub fade_out: bool,
    /// Randomize chosen foreground color.
    pub random_color: bool,
    /// Slide colors across instead of instant blink.
    pub slide_colors: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A color-change effect cycling through a fixed set of colors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectColorChange {
    /// Colors to cycle through.
    pub colors: ArrayVec<Color, 6>,

    /// Color change speed.
    pub speed: EffectPercent,

    /// Whether colors should fade smoothly.
    pub fade: bool,
    /// Randomize color order.
    pub random_color: bool,
    /// Slide between colors instead of jumping.
    pub slide_colors: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A sequence effect displaying multiple colors in order with delays.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectSequence {
    /// Background color.
    pub background: Color,
    /// Colors in the sequence.
    pub colors: ArrayVec<Color, 5>,

    /// Sequence speed.
    pub speed: EffectPercent,
    /// Transition smoothness between colors.
    pub smoothness: EffectPercent,
    /// Delay after finishing the sequence.
    pub delay_after_sequence: EffectDelay,
    /// Delay before starting the sequence.
    pub delay_before_sequence: EffectDelay,

    /// Play the sequence in reverse direction.
    pub reverse_direction: bool,
    /// Enable fading between sequence colors.
    pub fade: bool,
    /// Randomize sequence colors.
    pub random_color: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A scanner effect sweeping a light point across the LEDs.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct EffectScanner {
    /// Background color of the scan.
    pub background: Color,
    /// Inner beam color.
    pub inner_color: Color,
    /// Outer beam color.
    pub outer_color: Color,

    /// Scan speed.
    pub speed: EffectPercent,
    /// Smoothness of the beam edges.
    pub smoothness: EffectPercent,
    /// Beam width.
    pub width: EffectWidth,

    /// Reverse scan direction.
    pub reverse_direction: bool,
    /// Fade beam intensity at edges.
    pub fade: bool,
    /// Use random beam colors.
    pub random_color: bool,
    /// Enable second color mode.
    pub second_color_mode: bool,
    /// Change colors during scanning.
    pub color_change: bool,
    /// Wrap scanner in circular mode.
    pub circular: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A wave effect moving a multicolor pattern across LEDs.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectWave {
    /// Background color.
    pub background: Color,
    /// Colors forming the wave.
    pub colors: ArrayVec<Color, 5>,

    /// Wave speed.
    pub speed: EffectPercent,
    /// Smoothness of transitions.
    pub smoothness: EffectPercent,
    /// Wave width.
    pub width: EffectWidth,

    /// Reverse wave direction.
    pub reverse_direction: bool,
    /// Use random colors.
    pub random_color: bool,
    /// Circular wave mode.
    pub circular: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A color sequence effect shifting through defined colors with a set speed.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectColorSequence {
    /// Sequence of colors.
    pub colors: ArrayVec<Color, 6>,

    /// Sequence speed.
    pub speed: EffectPercent,
    /// Smoothness of transitions.
    pub smoothness: EffectPercent,
    /// Speed of color changes.
    pub color_change_speed: EffectWidth,

    /// Reverse sequence direction.
    pub reverse_direction: bool,
    /// Randomize sequence colors.
    pub random_color: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A color-shift effect scrolling colors across a given area.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectColorShift {
    /// Base color.
    pub color: Color,

    /// Shift speed.
    pub speed: EffectPercent,
    /// Color range within the shift.
    pub color_range: EffectPercent,
    /// Total affected area width.
    pub total_area: EffectWidth,

    /// Reverse shift direction.
    pub reverse_direction: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A bar graph effect mapping values to colored LED ranges.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct EffectBarGraph {
    /// Background color.
    pub background: Color,
    /// Color of the peak indicator.
    pub peak_color: Color,
    /// Value ranges mapped to colors (with thresholds and fades).
    pub colors: ArrayVec<(Color, u16, bool), 4>,

    /// Maximum value for the bar graph.
    pub end_value: u16,
    /// Graph rotation.
    pub rotation: EffectPercent,
    /// Duration to hold peak values.
    pub peak_hold_time: EffectPercent,

    /// Reverse bar growth direction.
    pub reverse_direction: bool,
    /// Show peak marker.
    pub show_peak: bool,
    /// Show bar itself.
    pub show_bar: bool,
    /// Show defined ranges.
    pub show_ranges: bool,
    /// Fade between ranges.
    pub fade_ranges: bool,

    /// Optional external control for rotation.
    pub source_control_rotation: Option<SourceControl>,
}

/// A flame-like randomized flickering effect.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectFlame {
    /// Background color.
    pub background: Color,
    /// Primary flame color.
    pub color_primary: Color,
    /// Secondary flame color.
    pub color_secondary: Color,

    /// Flame intensity (size/strength).
    pub intensity: EffectWidth,

    /// Optional external control for intensity.
    pub source_control_intensity: Option<SourceControl>,
}

/// A rain effect simulating falling drops.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectRain {
    /// Background color.
    pub background: Color,
    /// Drop color.
    pub color: Color,

    /// Falling speed.
    pub speed: EffectWidth,
    /// Number of falling drops.
    pub items: RainItems,
    /// Drop size.
    pub size: EffectWidth,
    /// Smoothness of falling motion.
    pub smoothness: EffectWidth,

    /// Reverse rain direction.
    pub reverse_direction: bool,
    /// Randomize drop colors.
    pub random_color: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A color switch effect cycling between defined color ranges.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectColorSwitch {
    /// Colors and ranges with thresholds.
    pub colors: ArrayVec<(Color, u16, bool), 6>,

    /// Maximum value for the switch.
    pub end_value: u16,

    /// Enable fading between ranges.
    pub fade_ranges: bool,

    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A swiping rainbow effect with a moving point and strip color.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectSwipingRainbow {
    /// Color of the moving point.
    pub point_color: Color,
    /// Color of the rainbow strip.
    pub strip_color: Color,

    /// Speed of the point movement.
    pub point_speed: EffectWidth,
    /// Smoothness of point edges.
    pub point_smoothness: EffectWidth,
    /// Size of the moving point.
    pub point_size: EffectWidth,
    /// Speed of rainbow color changes.
    pub color_change_speed: EffectWidth,
    /// Range of rainbow colors.
    pub color_range: EffectWidth,

    /// Reverse swipe direction.
    pub reverse_direction: bool,

    /// Optional external control for speed.
    pub source_control_speed: Option<SourceControl>,
    /// Optional external control for brightness.
    pub source_control_brightness: Option<SourceControl>,
}

/// A sound-reactive flash effect.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectSoundFlash {
    /// Background color.
    pub background: Color,
    /// Colors triggered by sound.
    pub colors: [Color; 4],
}

/// A sound-reactive slider effect.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectSoundSlider {
    /// Background color.
    pub background: Color,
    /// Color/effect/speed mapping for sliders.
    pub effects: [(Color, SoundEffect, SoundEffectSpeed); 4],
    /// Rotation speed for cycling colors.
    pub rotate_color: EffectPercent,
}

/// A sound-reactive shifting effect.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectSoundShift {
    /// Background color.
    pub background: Color,
    /// Color/effect-speed pairs with activity flag.
    pub effects: [(Color, SoundEffectSpeed, bool); 2],
    /// Speed of rotating colors.
    pub rotate_color: EffectPercent,
    /// Speed when idle.
    pub idle_speed: EffectPercent,
    /// Speed during activity.
    pub activity_speed: EffectPercent,
    /// Reverse shifting direction.
    pub reverse_direction: bool,
}

/// An ambient background effect.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectAmbient {
    /// Background color.
    pub background: Color,
}

/// A gradient effect blending multiple colors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EffectColorGradient {
    /// Starting color of the gradient.
    pub start_color: Color,
    /// Additional colors with positions.
    pub colors: ArrayVec<(Color, u16), 3>,

    /// Gradient rotation speed.
    pub rotation: EffectPercent,

    /// Reverse gradient direction.
    pub reverse_direction: bool,
    /// Reverse rotation of gradient.
    pub reverse_rotation: bool,

    /// Optional external control for rotation.
    pub source_control_rotation: Option<SourceControl>,
}

impl Decode for Option<Controller> {
    #[allow(clippy::too_many_lines)]
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let offset = reader.read_u8()?;
        let length = reader.read_u8()?;
        let effect = reader.read_u8()?;

        let flags = reader.read_u16be()?;
        let data_source = Option::<DataSource>::decode(reader)?;
        let sensor_attenuation_rising = reader.read_u8()?;
        let sensor_attenuation_falling = reader.read_u8()?;

        let sc0 = flag_set(flags, 0x4000);
        let sc1 = flag_set(flags, 0x8000);

        let effect = match effect {
            0x00 => {
                reader.skip::<61>()?;

                return Ok(R::guard(|_| None));
            }
            0x01 => {
                let source_control_brightness = SourceControl::decode_opt(reader, sc0)?;
                let source_control_saturation = SourceControl::decode_opt(reader, sc1)?;

                reader.skip::<24>()?;

                let color = Color::decode(reader)?;
                <[Color; 5]>::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::Static(EffectStatic {
                        color: x.extract(color),
                        source_control_brightness: x.extract(source_control_brightness),
                        source_control_saturation: x.extract(source_control_saturation),
                    })
                })
            }
            0x02 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_intensity = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let intensity = EffectPercent::decode(reader)?;
                let delay_max_brightness = EffectDelay::decode(reader)?;
                let delay_min_brightness = EffectDelay::decode(reader)?;
                reader.skip::<16>()?;

                let color = Color::decode(reader)?;
                <[Color; 5]>::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::Breathing(EffectBreathing {
                        color: x.extract(color),
                        speed: x.extract(speed),
                        intensity: x.extract(intensity),
                        delay_max_brightness: x.extract(delay_max_brightness),
                        delay_min_brightness: x.extract(delay_min_brightness),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_intensity: x.extract(source_control_intensity),
                    })
                })
            }
            0x03 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let color_range = EffectPercent::decode(reader)?;
                reader.skip::<20>()?;

                let color = Color::decode(reader)?;
                <[Color; 5]>::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::Rainbow(EffectRainbow {
                        color: x.extract(color),
                        speed: x.extract(speed),
                        color_range: x.extract(color_range),
                        reverse_direction: flag_set(flags, 0x02),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x04 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let color_count = usize::from(reader.read_u16be()?);
                reader.skip::<20>()?;

                let colors = <[Color; 6]>::decode(reader)?;

                R::guard(|x| {
                    let mut colors = x.extract(colors).into_iter();

                    Effect::Blink(EffectBlink {
                        background: colors.next().unwrap(),
                        colors: colors.take(color_count).collect(),
                        speed: x.extract(speed),
                        fade_in: flag_set(flags, 0x02),
                        fade_out: flag_set(flags, 0x04),
                        random_color: flag_set(flags, 0x08),
                        slide_colors: flag_set(flags, 0x10),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x05 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let color_count = usize::from(reader.read_u16be()?);
                reader.skip::<20>()?;

                let colors = <[Color; 6]>::decode(reader)?;

                R::guard(|x| {
                    Effect::ColorChange(EffectColorChange {
                        colors: x.extract(colors).into_iter().take(color_count).collect(),
                        speed: x.extract(speed),
                        fade: flag_set(flags, 0x04),
                        random_color: flag_set(flags, 0x08),
                        slide_colors: flag_set(flags, 0x10),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x07 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let smoothness = EffectPercent::decode(reader)?;
                let color_count = usize::from(reader.read_u16be()?);
                let delay_after_sequence = EffectDelay::decode(reader)?;
                let delay_before_sequence = EffectDelay::decode(reader)?;
                reader.skip::<14>()?;

                let colors = <[Color; 6]>::decode(reader)?;

                R::guard(|x| {
                    let mut colors = x.extract(colors).into_iter();

                    Effect::Sequence(EffectSequence {
                        background: colors.next().unwrap(),
                        colors: colors.take(color_count).collect(),
                        speed: x.extract(speed),
                        smoothness: x.extract(smoothness),
                        delay_after_sequence: x.extract(delay_after_sequence),
                        delay_before_sequence: x.extract(delay_before_sequence),
                        reverse_direction: flag_set(flags, 0x02),
                        fade: flag_set(flags, 0x04),
                        random_color: flag_set(flags, 0x08),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x08 | 0x09 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let smoothness = EffectPercent::decode(reader)?;
                let width = EffectWidth::decode(reader)?;
                reader.skip::<18>()?;

                let background = Color::decode(reader)?;
                let outer_color = Color::decode(reader)?;
                let inner_color = Color::decode(reader)?;
                <[Color; 3]>::skip_bytes(reader)?;

                R::guard(|x| {
                    let data = EffectScanner {
                        background: x.extract(background),
                        inner_color: x.extract(inner_color),
                        outer_color: x.extract(outer_color),
                        speed: x.extract(speed),
                        smoothness: x.extract(smoothness),
                        width: x.extract(width),
                        reverse_direction: flag_set(flags, 0x02),
                        fade: flag_set(flags, 0x04),
                        random_color: flag_set(flags, 0x08),
                        second_color_mode: flag_set(flags, 0x20),
                        color_change: flag_set(flags, 0x40),
                        circular: flag_set(flags, 0x80),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    };

                    match effect {
                        0x08 => Effect::Scanner(data),
                        0x09 => Effect::Laser(data),
                        _ => unreachable!(),
                    }
                })
            }
            0x0A => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let smoothness = EffectPercent::decode(reader)?;
                let width = EffectWidth::decode(reader)?;
                let color_count = usize::from(reader.read_u16be()?);
                reader.skip::<16>()?;

                let colors = <[Color; 6]>::decode(reader)?;

                R::guard(|x| {
                    let mut colors = x.extract(colors).into_iter();

                    Effect::Wave(EffectWave {
                        background: colors.next().unwrap(),
                        colors: colors.take(color_count).collect(),
                        speed: x.extract(speed),
                        smoothness: x.extract(smoothness),
                        width: x.extract(width),
                        reverse_direction: flag_set(flags, 0x02),
                        random_color: flag_set(flags, 0x04),
                        circular: flag_set(flags, 0x80),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x0B => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let smoothness = EffectPercent::decode(reader)?;
                reader.skip::<2>()?;
                let color_count = usize::from(reader.read_u16be()?);
                let color_change_speed = EffectWidth::decode(reader)?;
                reader.skip::<14>()?;

                let colors = <[Color; 6]>::decode(reader)?;

                R::guard(|x| {
                    Effect::ColorSequence(EffectColorSequence {
                        colors: x.extract(colors).into_iter().take(color_count).collect(),
                        speed: x.extract(speed),
                        smoothness: x.extract(smoothness),
                        color_change_speed: x.extract(color_change_speed),
                        reverse_direction: flag_set(flags, 0x02),
                        random_color: flag_set(flags, 0x08),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x0C => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectPercent::decode(reader)?;
                let color_range = EffectPercent::decode(reader)?;
                let total_area = EffectWidth::decode(reader)?;
                reader.skip::<18>()?;

                let color = Color::decode(reader)?;
                <[Color; 5]>::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::ColorShift(EffectColorShift {
                        color: x.extract(color),
                        speed: x.extract(speed),
                        color_range: x.extract(color_range),
                        total_area: x.extract(total_area),
                        reverse_direction: flag_set(flags, 0x02),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x0D | 0x15 => {
                let source_control_rotation = SourceControl::decode_opt(reader, sc0)?;
                SourceControl::skip_bytes(reader)?;

                let start_value = reader.read_u16be()?;
                let end_value = reader.read_u16be()?;
                let rotation = EffectPercent::decode(reader)?;
                let color_count = usize::from(reader.read_u16be()?);
                let color_values = <[u16; 3]>::decode(reader)?;
                let peak_hold_time = EffectPercent::decode(reader)?;
                reader.skip::<8>()?;

                let background = Color::decode(reader)?;
                let peak_color = Color::decode(reader)?;
                let colors = <[Color; 4]>::decode(reader)?;

                R::guard(|x| {
                    let colors = x.extract(colors).into_iter();
                    let values = Some(start_value).into_iter().chain(x.extract(color_values));

                    let colors = colors
                        .zip(values)
                        .enumerate()
                        .map(|(index, (color, value))| {
                            let flag = index + 7;
                            let blink = flag_set(flags, 1 << flag);

                            (color, value, blink)
                        })
                        .take(color_count + 1)
                        .collect();

                    let data = EffectBarGraph {
                        background: x.extract(background),
                        peak_color: x.extract(peak_color),
                        colors,

                        end_value,
                        rotation: x.extract(rotation),
                        peak_hold_time: x.extract(peak_hold_time),

                        reverse_direction: flag_set(flags, 0x08),
                        show_peak: flag_set(flags, 0x20),
                        show_bar: flag_set(flags, 0x02),
                        show_ranges: flag_set(flags, 0x04),
                        fade_ranges: flag_set(flags, 0x01),

                        source_control_rotation: x.extract(source_control_rotation),
                    };

                    match effect {
                        0x0D => Effect::BarGraph(data),
                        0x15 => Effect::SoundBars(data),
                        _ => unreachable!(),
                    }
                })
            }
            0x0E => {
                let source_control_intensity = SourceControl::decode_opt(reader, sc0)?;
                SourceControl::skip_bytes(reader)?;

                let intensity = EffectWidth::decode(reader)?;
                reader.skip::<22>()?;

                let background = Color::decode(reader)?;
                let color_primary = Color::decode(reader)?;
                let color_secondary = Color::decode(reader)?;
                <[Color; 3]>::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::Flame(EffectFlame {
                        background: x.extract(background),
                        color_primary: x.extract(color_primary),
                        color_secondary: x.extract(color_secondary),
                        intensity: x.extract(intensity),
                        source_control_intensity: x.extract(source_control_intensity),
                    })
                })
            }
            0x0F..=0x11 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let speed = EffectWidth::decode(reader)?;
                let items = RainItems::decode(reader)?;
                let size = EffectWidth::decode(reader)?;
                let smoothness = EffectWidth::decode(reader)?;
                reader.skip::<16>()?;

                let background = Color::decode(reader)?;
                let color = Color::decode(reader)?;
                <[Color; 4]>::skip_bytes(reader)?;

                R::guard(|x| {
                    let rain = EffectRain {
                        background: x.extract(background),
                        color: x.extract(color),
                        speed: x.extract(speed),
                        items: x.extract(items),
                        size: x.extract(size),
                        smoothness: x.extract(smoothness),
                        random_color: flag_set(flags, 0x08),
                        reverse_direction: flag_set(flags, 0x02),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    };

                    match effect {
                        0x0F => Effect::Rain(rain),
                        0x10 => Effect::Snow(rain),
                        0x11 => Effect::Stardust(rain),
                        _ => unreachable!(),
                    }
                })
            }
            0x12 => {
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;
                SourceControl::skip_bytes(reader)?;

                let color_count = usize::from(reader.read_u16be()?);
                let values = <[u16; 6]>::decode(reader)?;
                let end_value = reader.read_u16be()?;
                reader.skip::<8>()?;

                let colors = <[Color; 6]>::decode(reader)?;

                R::guard(|x| {
                    let values = x.extract(values).into_iter();
                    let colors = x.extract(colors).into_iter();

                    let colors = colors
                        .zip(values)
                        .enumerate()
                        .map(|(index, (color, value))| {
                            let flag = index + 1;
                            let blink = flag_set(flags, 1 << flag);

                            (color, value, blink)
                        })
                        .take(color_count + 1)
                        .collect();

                    Effect::ColorSwitch(EffectColorSwitch {
                        colors,
                        end_value,
                        fade_ranges: flag_set(flags, 0x01),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x13 => {
                let source_control_speed = SourceControl::decode_opt(reader, sc0)?;
                let source_control_brightness = SourceControl::decode_opt(reader, sc1)?;

                let point_speed = EffectWidth::decode(reader)?;
                let point_smoothness = EffectWidth::decode(reader)?;
                let point_size = EffectWidth::decode(reader)?;
                let color_change_speed = EffectWidth::decode(reader)?;
                let color_range = EffectWidth::decode(reader)?;
                reader.skip::<14>()?;

                let point_color = Color::decode(reader)?;
                let strip_color = Color::decode(reader)?;
                <[Color; 4]>::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::SwipingRainbow(EffectSwipingRainbow {
                        point_color: x.extract(point_color),
                        strip_color: x.extract(strip_color),
                        point_speed: x.extract(point_speed),
                        point_smoothness: x.extract(point_smoothness),
                        point_size: x.extract(point_size),
                        color_change_speed: x.extract(color_change_speed),
                        color_range: x.extract(color_range),
                        reverse_direction: flag_set(flags, 0x01),
                        source_control_speed: x.extract(source_control_speed),
                        source_control_brightness: x.extract(source_control_brightness),
                    })
                })
            }
            0x14 => {
                SourceControl::skip_bytes(reader)?;
                SourceControl::skip_bytes(reader)?;

                reader.skip::<24>()?;

                let background = Color::decode(reader)?;
                let colors = <[Color; 4]>::decode(reader)?;
                Color::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::SoundFlash(EffectSoundFlash {
                        background: x.extract(background),
                        colors: x.extract(colors),
                    })
                })
            }
            0x16 => {
                SourceControl::skip_bytes(reader)?;
                SourceControl::skip_bytes(reader)?;

                let effects = <[SoundEffect; 4]>::decode(reader)?;
                let speeds = <[SoundEffectSpeed; 4]>::decode(reader)?;
                let rotate_color = EffectPercent::decode(reader)?;
                reader.skip::<6>()?;

                let background = Color::decode(reader)?;
                let colors = <[Color; 4]>::decode(reader)?;
                Color::skip_bytes(reader)?;

                R::guard(|x| {
                    let effects = x.extract(effects).into_iter();
                    let speeds = x.extract(speeds).into_iter();
                    let colors = x.extract(colors).into_iter();

                    let mut effects = effects
                        .zip(speeds)
                        .zip(colors)
                        .map(|((effect, speed), color)| (color, effect, speed));

                    Effect::SoundSlider(EffectSoundSlider {
                        background: x.extract(background),
                        effects: from_fn(|_| effects.next().unwrap()),
                        rotate_color: x.extract(rotate_color),
                    })
                })
            }
            0x17 => {
                SourceControl::skip_bytes(reader)?;
                SourceControl::skip_bytes(reader)?;

                let rotate_color = EffectPercent::decode(reader)?;
                let effects = <[SoundEffectSpeed; 2]>::decode(reader)?;
                let idle_speed = EffectPercent::decode(reader)?;
                let activity_speed = EffectPercent::decode(reader)?;
                reader.skip::<14>()?;

                let background = Color::decode(reader)?;
                let colors = <[Color; 2]>::decode(reader)?;
                <[Color; 3]>::skip_bytes(reader)?;

                R::guard(|x| {
                    let effects = x.extract(effects).into_iter();
                    let colors = x.extract(colors).into_iter();

                    let mut effects =
                        effects
                            .zip(colors)
                            .enumerate()
                            .map(|(index, (effect, color))| {
                                let flag = index + 1;
                                let random_color = flag_set(flags, 1 << flag);

                                (color, effect, random_color)
                            });

                    Effect::SoundShift(EffectSoundShift {
                        background: x.extract(background),
                        effects: from_fn(|_| effects.next().unwrap()),
                        rotate_color: x.extract(rotate_color),
                        idle_speed: x.extract(idle_speed),
                        activity_speed: x.extract(activity_speed),
                        reverse_direction: flag_set(flags, 0x01),
                    })
                })
            }
            0x18 => {
                SourceControl::skip_bytes(reader)?;
                SourceControl::skip_bytes(reader)?;

                reader.skip::<24>()?;

                let background = Color::decode(reader)?;
                <[Color; 5]>::skip_bytes(reader)?;

                R::guard(|x| {
                    Effect::Ambient(EffectAmbient {
                        background: x.extract(background),
                    })
                })
            }
            0x21 => {
                let source_control_rotation = SourceControl::decode_opt(reader, sc0)?;
                SourceControl::skip_bytes(reader)?;

                reader.skip::<4>()?;
                let rotation = EffectPercent::decode(reader)?;
                let color_count = usize::from(reader.read_u16be()?);
                let values = <[u16; 3]>::decode(reader)?;
                reader.skip::<10>()?;

                <[Color; 2]>::skip_bytes(reader)?;
                let start_color = Color::decode(reader)?;
                let colors = <[Color; 3]>::decode(reader)?;

                R::guard(|x| {
                    let colors = x.extract(colors).into_iter();
                    let values = x.extract(values).into_iter();

                    let colors = colors.zip(values).take(color_count).collect();

                    Effect::ColorGradient(EffectColorGradient {
                        start_color: x.extract(start_color),
                        colors,

                        rotation: x.extract(rotation),

                        reverse_direction: flag_set(flags, 0x08),
                        reverse_rotation: flag_set(flags, 0x10),

                        source_control_rotation: x.extract(source_control_rotation),
                    })
                })
            }
            x => Err(IoError::InvalidValue("Effect", x.into()))?,
        };

        reader.skip::<1>()?;

        Ok(R::guard(|x| {
            Some(Controller {
                offset,
                length,
                data_source: x.extract(data_source),
                sensor_attenuation_rising,
                sensor_attenuation_falling,
                effect: x.extract(effect),
            })
        }))
    }
}

/// A mapping configuration that scales an external input signal into an effect parameter range.
///
/// The actual values of `input_min` and `input_max` depend on the selected data source
/// (e.g. temperature, flow rate, sensor value).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SourceControl {
    /// Minimum expected value of the input signal (depends on the data source).
    pub input_min: u16,
    /// Maximum expected value of the input signal (depends on the data source).
    pub input_max: u16,
    /// Minimum output value mapped to the effect parameter.
    pub output_min: u8,
    /// Maximum output value mapped to the effect parameter.
    pub output_max: u8,
}

impl Decode for SourceControl {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let input_min = reader.read_u16be()?;
        let input_max = reader.read_u16be()?;
        let output_min = reader.read_u8()?;
        let output_max = reader.read_u8()?;

        Ok(R::guard(|_| Self {
            input_min,
            input_max,
            output_min,
            output_max,
        }))
    }
}
/// A wrapper around [`Hsv`] representing a color used in effects.
///
/// Provides convenience constructors from HSV, RGB, and hexadecimal RGB values.
#[derive(Debug, Clone)]
pub struct Color(pub Hsv);

impl Color {
    /// Creates a [`Color`] directly from HSV components.
    ///
    /// - `h`: Hue, usually in degrees `[0.0 .. 360.0)`.
    /// - `s`: Saturation `[0.0 .. 1.0]`.
    /// - `v`: Value (brightness) `[0.0 .. 1.0]`.
    #[must_use]
    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        Self(Hsv::new(h, s, v))
    }

    /// Creates a [`Color`] from 8-bit RGB components.
    ///
    /// - `r`: Red channel `[0 .. 255]`.
    /// - `g`: Green channel `[0 .. 255]`.
    /// - `b`: Blue channel `[0 .. 255]`.
    #[must_use]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(Hsv::from_rgb(&Rgb::new(
            f64::from(r),
            f64::from(g),
            f64::from(b),
        )))
    }

    /// Creates a [`Color`] from a packed hexadecimal RGB value.
    ///
    /// The `hex` value should be in the form `0xRRGGBB`.
    ///
    /// Example:
    /// ```rust
    /// use high_flow_next::protocol::settings::Color;
    ///
    /// let c = Color::from_rgb_hex(0xFF8800); // orange
    /// ```
    #[must_use]
    pub fn from_rgb_hex(hex: u32) -> Self {
        Self(Hsv::from_rgb(&Rgb::from_hex(hex)))
    }
}

impl From<Hsv> for Color {
    fn from(value: Hsv) -> Self {
        Self(value)
    }
}

impl From<Color> for Hsv {
    fn from(value: Color) -> Self {
        value.0
    }
}

impl Eq for Color {}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.0.h.eq(&other.0.h) && self.0.s.eq(&other.0.s) && self.0.v.eq(&other.0.v)
    }
}

impl Decode for Color {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let h_section = f64::from(reader.read_u8()?);
        let h_offset = f64::from(reader.read_u8()?);
        let s = f64::from(reader.read_u8()?);
        let v = f64::from(reader.read_u8()?);

        Ok(R::guard(|_| {
            Self::from_hsv(
                60.0 * h_section + 60.0 * h_offset / 255.0,
                s / 255.0,
                v / 255.0,
            )
        }))
    }
}
/// Represents the origin of a control signal that can be mapped into an effect
/// parameter using [`SourceControl`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DataSource {
    /// Flow rate measured by the device.
    Flow,
    /// Water temperature sensor value.
    WaterTemperature,
    /// External (ambient) temperature sensor value.
    ExternalTemperature,
    /// Electrical conductivity of the coolant.
    Conductivity,
    /// Water quality index (based on conductivity and other factors).
    WaterQuality,
    /// Power consumption measurement.
    Power,
    /// Sound input or sound level measurement.
    Sound,
    /// Virtual software-defined sensor channel 1.
    SoftwareSensor1,
    /// Virtual software-defined sensor channel 2.
    SoftwareSensor2,
    /// Virtual software-defined sensor channel 3.
    SoftwareSensor3,
    /// Virtual software-defined sensor channel 4.
    SoftwareSensor4,
    /// Virtual software-defined sensor channel 5.
    SoftwareSensor5,
    /// Virtual software-defined sensor channel 6.
    SoftwareSensor6,
    /// Virtual software-defined sensor channel 7.
    SoftwareSensor7,
    /// Virtual software-defined sensor channel 8.
    SoftwareSensor8,
}

impl Decode for Option<DataSource> {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u16be()? {
            0x0000 => Ok(R::guard(|_| Some(DataSource::Flow))),
            0x0001 => Ok(R::guard(|_| Some(DataSource::WaterTemperature))),
            0x0002 => Ok(R::guard(|_| Some(DataSource::ExternalTemperature))),
            0x0003 => Ok(R::guard(|_| Some(DataSource::Conductivity))),
            0x0004 => Ok(R::guard(|_| Some(DataSource::WaterQuality))),
            0x0005 => Ok(R::guard(|_| Some(DataSource::Power))),
            0x0006 => Ok(R::guard(|_| Some(DataSource::SoftwareSensor1))),
            0x0007 => Ok(R::guard(|_| Some(DataSource::SoftwareSensor2))),
            0x0008 => Ok(R::guard(|_| Some(DataSource::SoftwareSensor3))),
            0x0009 => Ok(R::guard(|_| Some(DataSource::SoftwareSensor4))),
            0x000A => Ok(R::guard(|_| Some(DataSource::SoftwareSensor5))),
            0x000B => Ok(R::guard(|_| Some(DataSource::SoftwareSensor6))),
            0x000C => Ok(R::guard(|_| Some(DataSource::SoftwareSensor7))),
            0x000D => Ok(R::guard(|_| Some(DataSource::SoftwareSensor8))),
            0x001C => Ok(R::guard(|_| Some(DataSource::Sound))),
            0xFFFF => Ok(R::guard(|_| None)),
            x => Err(IoError::InvalidValue("DataSource", x.into())),
        }
    }
}

/// Defines how LEDs should react spatially to sound input.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SoundEffect {
    /// Expands outward from the center of the LED strip or area.
    OutwardsFromCenter,
    /// Contracts inward toward the center (mode A).
    InwardsToCenterA,
    /// Contracts inward toward the center (mode B, alternative pattern).
    InwardsToCenterB,
    /// Animates in response to sound from the left side.
    FromLeft,
    /// Animates in response to sound from the right side.
    FromRight,
    /// Affects all LEDs simultaneously with the sound.
    AllLEDs,
}

impl Decode for SoundEffect {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        match reader.read_u16be()? {
            0x0000 => Ok(R::guard(|_| SoundEffect::OutwardsFromCenter)),
            0x0001 => Ok(R::guard(|_| SoundEffect::InwardsToCenterA)),
            0x0002 => Ok(R::guard(|_| SoundEffect::InwardsToCenterB)),
            0x0003 => Ok(R::guard(|_| SoundEffect::FromLeft)),
            0x0004 => Ok(R::guard(|_| SoundEffect::FromRight)),
            0x0005 => Ok(R::guard(|_| SoundEffect::AllLEDs)),
            x => Err(IoError::InvalidValue("SoundEffect", x.into())),
        }
    }
}

define_wrapped! {
    /// Brightness level of an effect or static color.
    ///
    /// Valid value range: 0..=255
    pub type Brightness<u8, BrightnessTag>;
}
impl_verify_simple!(Brightness<u8, BrightnessTag>);

define_wrapped! {
    /// General-purpose percentage value used for effect speed, intensity, and similar parameters.
    ///
    /// Valid value range: 0..=100 (%)
    pub type EffectPercent<u16, EffectPercentTag>;
}
impl_ranged!(EffectPercent<u16, EffectPercentTag>, 0, 100);

define_wrapped! {
    /// Delay time used in effects (e.g. breathing min/max delays, sequence waits).
    ///
    /// Valid value range: 0..=100 (units depend on context, typically milliseconds or percent scale)
    pub type EffectDelay<u16, EffectDelayTag>;
}
impl_ranged!(EffectDelay<u16, EffectDelayTag>, 0, 100);

define_wrapped! {
    /// Width parameter for effects (e.g. scanner beam width, wave width, flame intensity).
    ///
    /// Valid value range: 1..=100 (relative width or intensity scale)
    pub type EffectWidth<u16, EffectWidthTag>;
}
impl_ranged!(EffectWidth<u16, EffectWidthTag>, 1, 100);

define_wrapped! {
    /// Number of raindrop items in the rain effect.
    ///
    /// Valid value range: 1..=4
    pub type RainItems<u16, RainItemsTag>;
}
impl_ranged!(RainItems<u16, RainItemsTag>, 1, 4);

define_wrapped! {
    /// Speed scaling factor for sound-reactive effects.
    ///
    /// Valid value range: 1..=10
    pub type SoundEffectSpeed<u16, SoundEffectSpeedTag>;
}
impl_ranged!(SoundEffectSpeed<u16, SoundEffectSpeedTag>, 1, 10);
