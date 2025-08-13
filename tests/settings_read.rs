#![allow(missing_docs, clippy::too_many_lines, clippy::unreadable_literal)]

use std::fs::File;

use high_flow_next::{
    misc::Decode,
    protocol::{
        settings::{
            AlarmFlags, Chart, ChartInterval, ChartSource, Color, ConnectorType, DataSource,
            DisplayBrightness, DisplayFlags, Effect, Flow, FlowCorrection, FlowUnit, Medium,
            OutputSignal, PageFlags, PowerFlags, SoundEffect, SoundEffectSpeed, SourceControl,
            StandbyFlags, Temperature, TemperatureUnit,
        },
        Frame,
    },
};

#[test]
fn default() {
    let mut reader = File::open("tests/assets/default.frame").unwrap();
    let frame = Frame::decode(&mut reader).unwrap();

    let Frame::Settings(values) = frame;

    macro_rules! flow_correction {
        ($flow:expr, $value:expr) => {
            (
                Flow::from_value($flow).unwrap(),
                FlowCorrection::from_value($value).unwrap(),
            )
        };
    }

    macro_rules! chart {
        ($source:ident, $interval:expr) => {
            Chart {
                source: ChartSource::$source,
                interval: ChartInterval::from_value($interval).unwrap(),
            }
        };
    }

    /* System */

    assert_eq!(values.system.standby_flags, StandbyFlags::empty());
    assert_eq!(*values.system.aqua_bus_address, 58);
    assert_eq!(values.system.increased_current_draw, None);

    /* Sensor */

    assert_eq!(values.sensor.medium, Medium::DpUltra);
    assert_eq!(
        values.sensor.connector_type,
        ConnectorType::InnerDiameterGt7mm
    );
    assert_eq!(
        values.sensor.flow_correction,
        [
            flow_correction!(200, 0),
            flow_correction!(300, 0),
            flow_correction!(500, 0),
            flow_correction!(700, 0),
            flow_correction!(1000, 0),
            flow_correction!(1250, 0),
            flow_correction!(1500, 0),
            flow_correction!(2000, 0),
            flow_correction!(2500, 0),
            flow_correction!(3000, 0)
        ]
    );

    assert_eq!(*values.sensor.water_temp_offset, 0);
    assert_eq!(*values.sensor.external_temp_offset, 0);

    assert_eq!(*values.sensor.conductivity_offset, 0);
    assert_eq!(*values.sensor.water_quality_max, 500);
    assert_eq!(*values.sensor.water_quality_min, 950);

    assert_eq!(values.sensor.power_flags, PowerFlags::empty());
    assert_eq!(*values.sensor.power_damping, 0);

    /* Alarms */

    assert_eq!(
        values.alarms.flags,
        AlarmFlags::ENABLE_ACUSTIC_INDICATOR
            | AlarmFlags::ENABLE_OPTICAL_INDICATOR
            | AlarmFlags::DISABLE_SIGNAL_OUTPUT_DURING_ALARM
    );
    assert_eq!(*values.alarms.startup_delay, 10);
    assert_eq!(values.alarms.flow_alarm_limit, None);
    assert_eq!(
        values.alarms.water_temperature_limit,
        Some(Temperature::from_value(4500).unwrap())
    );
    assert_eq!(values.alarms.external_temperature_limit, None);
    assert_eq!(values.alarms.water_quality_limit, None);
    assert_eq!(values.alarms.output_signal, OutputSignal::ConstantSpeed);

    /* Display Settings */

    assert_eq!(values.display.temperature_unit, TemperatureUnit::C);
    assert_eq!(values.display.flow_unit, FlowUnit::Liter);
    assert_eq!(values.display.display_flags, DisplayFlags::AUTO_INVERT);
    assert_eq!(values.display.next_page_interval.as_deref(), Some(&10));
    assert_eq!(
        values.display.page_flags,
        PageFlags::DEVICE_INFO
            | PageFlags::FLOW
            | PageFlags::WATER_TEMP
            | PageFlags::CONDUCTIVITY
            | PageFlags::WATER_QUALITY
            | PageFlags::FLOW_WATERTEMP
            | PageFlags::COND_QUALITY
            | PageFlags::FLOW_VOLUME
            | PageFlags::CHART1
            | PageFlags::CHART2
            | PageFlags::CHART3
            | PageFlags::CHART4
    );
    assert_eq!(values.display.display_brightness, DisplayBrightness::Low);
    assert_eq!(
        values.display.idle_display_brightness,
        Some(DisplayBrightness::Low)
    );
    assert_eq!(
        values.display.charts,
        [
            chart!(Flow, 10),
            chart!(WaterTemp, 10),
            chart!(WaterQuality, 10),
            chart!(PowerConsumption, 10)
        ]
    );

    /* Lighting */

    let Some(lighting) = values.lighting else {
        panic!("Lighting is expected to be enabled!");
    };

    let mut strip = lighting.strip_controllers.into_iter();
    let mut sensor = lighting.sensor_controllers.into_iter();
    assert_eq!(*lighting.brightness, 255);

    // Controller 1 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 15);
    assert_eq!(controller.sensor_attenuation_falling, 25);
    let Effect::Rainbow(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.color, Color::from_rgb_hex(0x3C0000));
    assert_eq!(*effect.speed, 50);
    assert_eq!(*effect.color_range, 100);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);

    // Controller 2 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 15);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 15);
    assert_eq!(controller.sensor_attenuation_falling, 25);
    let Effect::Scanner(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.background, Color::from_rgb_hex(0x0F0F0F));
    assert_eq!(
        effect.inner_color,
        Color::from_hsv(59.76470588235294, 1.0, 1.0)
    );
    assert_eq!(
        effect.outer_color,
        Color::from_hsv(234.35294117647058, 1.0, 1.0)
    );
    assert_eq!(*effect.speed, 25);
    assert_eq!(*effect.smoothness, 40);
    assert_eq!(*effect.width, 20);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.reverse_direction);
    assert!(!effect.fade);
    assert!(!effect.random_color);
    assert!(!effect.second_color_mode);
    assert!(!effect.color_change);
    assert!(!effect.circular);

    // Controller 3 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 30);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 15);
    assert_eq!(controller.sensor_attenuation_falling, 25);
    let Effect::ColorSequence(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        &effect.colors[..],
        &[
            Color::from_hsv(180.0, 0.00784313725490196, 1.0),
            Color::from_hsv(119.05882352941177, 1.0, 1.0),
            Color::from_hsv(308.70588235294116, 1.0, 1.0),
            Color::from_hsv(46.35294117647059, 1.0, 1.0),
            Color::from_hsv(237.64705882352942, 1.0, 1.0),
            Color::from_hsv(357.1764705882353, 1.0, 1.0),
        ][..]
    );
    assert_eq!(*effect.speed, 30);
    assert_eq!(*effect.smoothness, 40);
    assert_eq!(*effect.color_change_speed, 80);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.reverse_direction);
    assert!(!effect.random_color);

    // Controller 4 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 45);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 15);
    assert_eq!(controller.sensor_attenuation_falling, 25);
    let Effect::Blink(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.background, Color::from_rgb_hex(0x0F0F0F));
    assert_eq!(
        &effect.colors[..],
        &[
            Color::from_hsv(0.0, 1.0, 1.0),
            Color::from_hsv(119.52941176470588, 1.0, 1.0),
            Color::from_hsv(240.0, 1.0, 1.0),
            Color::from_hsv(58.8235294117647, 1.0, 1.0),
            Color::from_hsv(108.47058823529412, 0.06274509803921569, 1.0),
        ][..]
    );
    assert_eq!(*effect.speed, 40);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(effect.fade_in);
    assert!(effect.fade_out);
    assert!(!effect.random_color);
    assert!(!effect.slide_colors);

    // Controller 5 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 60);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 15);
    assert_eq!(controller.sensor_attenuation_falling, 25);
    let Effect::Rainbow(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.color, Color::from_rgb_hex(0xFF0000));
    assert_eq!(*effect.speed, 50);
    assert_eq!(*effect.color_range, 100);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);

    // Controller 6 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 75);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 15);
    assert_eq!(controller.sensor_attenuation_falling, 25);
    let Effect::Rainbow(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.color, Color::from_rgb_hex(0xFF0000));
    assert_eq!(*effect.speed, 50);
    assert_eq!(*effect.color_range, 100);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);

    assert!(strip.next().is_none());

    // Controller 7 (Sensor)
    let controller = sensor.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 10);
    assert_eq!(controller.data_source, Some(DataSource::Flow));
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Wave(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(182.8235294117647, 1.0, 0.2)
    );
    assert_eq!(
        &effect.colors[..],
        &[Color::from_hsv(32.705882352941174, 1.0, 1.0)][..]
    );
    assert_eq!(*effect.speed, 7);
    assert_eq!(*effect.smoothness, 6);
    assert_eq!(*effect.width, 4);
    assert_eq!(
        effect.source_control_speed,
        Some(SourceControl {
            input_min: 0,
            input_max: 150,
            output_min: 0,
            output_max: 30,
        })
    );
    assert_eq!(effect.source_control_brightness, None);
    assert!(effect.reverse_direction);
    assert!(!effect.random_color);
    assert!(effect.circular);

    assert!(sensor.next().is_none());
}

#[test]
fn effects_0() {
    let mut reader = File::open("tests/assets/effects_0.frame").unwrap();
    let frame = Frame::decode(&mut reader).unwrap();

    let Frame::Settings(values) = frame;

    macro_rules! flow_correction {
        ($flow:expr, $value:expr) => {
            (
                Flow::from_value($flow).unwrap(),
                FlowCorrection::from_value($value).unwrap(),
            )
        };
    }

    macro_rules! chart {
        ($source:ident, $interval:expr) => {
            Chart {
                source: ChartSource::$source,
                interval: ChartInterval::from_value($interval).unwrap(),
            }
        };
    }

    /* System */

    assert_eq!(
        values.system.standby_flags,
        StandbyFlags::DISABLE_ALARM_DETECT
            | StandbyFlags::DISPLAY_OFF
            | StandbyFlags::LEDS_DISABLED
            | StandbyFlags::DISABLE_VOLUME_COUNTER
    );
    assert_eq!(*values.system.aqua_bus_address, 58);
    assert_eq!(values.system.increased_current_draw.as_deref(), Some(&600));

    /* Sensor */

    assert_eq!(values.sensor.medium, Medium::DistilledWater);
    assert_eq!(
        values.sensor.connector_type,
        ConnectorType::InnerDiameterLt7mm
    );
    assert_eq!(
        values.sensor.flow_correction,
        [
            flow_correction!(200, 1000),
            flow_correction!(300, -1000),
            flow_correction!(500, 500),
            flow_correction!(700, -500),
            flow_correction!(1000, 1500),
            flow_correction!(1250, -1523),
            flow_correction!(1500, 2512),
            flow_correction!(2000, -2579),
            flow_correction!(2500, 3033),
            flow_correction!(3000, -3333)
        ]
    );

    assert_eq!(*values.sensor.water_temp_offset, -51);
    assert_eq!(*values.sensor.external_temp_offset, 1055);

    assert_eq!(*values.sensor.conductivity_offset, 123);
    assert_eq!(*values.sensor.water_quality_max, 453);
    assert_eq!(*values.sensor.water_quality_min, 963);

    assert_eq!(
        values.sensor.power_flags,
        PowerFlags::AUTOMATIC_POWER_OFFSET_COMPENSATION
    );
    assert_eq!(*values.sensor.power_damping, 616);

    /* Alarms */

    assert_eq!(
        values.alarms.flags,
        AlarmFlags::ENABLE_ACUSTIC_INDICATOR | AlarmFlags::ENABLE_OPTICAL_INDICATOR
    );
    assert_eq!(*values.alarms.startup_delay, 10);
    assert_eq!(values.alarms.flow_alarm_limit, None);
    assert_eq!(
        values.alarms.water_temperature_limit.as_deref(),
        Some(&4510)
    );
    assert_eq!(
        values.alarms.external_temperature_limit.as_deref(),
        Some(&5680)
    );
    assert_eq!(values.alarms.water_quality_limit.as_deref(), Some(&3329));
    assert_eq!(values.alarms.output_signal, OutputSignal::PermanentOn);

    /* Display Settings */

    assert_eq!(values.display.temperature_unit, TemperatureUnit::F);
    assert_eq!(values.display.flow_unit, FlowUnit::Liter);
    assert_eq!(
        values.display.display_flags,
        DisplayFlags::ROTATE | DisplayFlags::DISABLE_BUTTONS
    );
    assert_eq!(values.display.next_page_interval, None);
    assert_eq!(
        values.display.page_flags,
        PageFlags::FLOW_WATERTEMP
            | PageFlags::COND_QUALITY
            | PageFlags::TEMPERATURES
            | PageFlags::FLOW_VOLUME
    );
    assert_eq!(
        values.display.display_brightness,
        DisplayBrightness::Maximum
    );
    assert_eq!(values.display.idle_display_brightness, None);
    assert_eq!(
        values.display.charts,
        [
            chart!(SystemVoltage, 1),
            chart!(Conductivity, 50),
            chart!(ExternalTemp, 100),
            chart!(WaterTemp, 600),
        ]
    );

    /* Lighting */

    let Some(lighting) = values.lighting else {
        panic!("Lighting is expected to be enabled!");
    };

    let mut strip = lighting.strip_controllers.into_iter();
    let mut sensor = lighting.sensor_controllers.into_iter();
    assert_eq!(*lighting.brightness, 230);

    // Controller 1 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Static(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.color,
        Color::from_hsv(72.94117647058823, 0.5882352941176471, 1.0)
    );
    assert_eq!(effect.source_control_brightness, None);
    assert_eq!(effect.source_control_saturation, None);

    // Controller 2 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 15);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Breathing(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.color,
        Color::from_hsv(314.11764705882354, 0.6196078431372549, 1.0)
    );
    assert_eq!(*effect.speed, 48);
    assert_eq!(*effect.intensity, 72);
    assert_eq!(*effect.delay_max_brightness, 18);
    assert_eq!(*effect.delay_min_brightness, 30);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_intensity, None);

    // Controller 3 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 30);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, Some(DataSource::WaterQuality));
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::ColorChange(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        &effect.colors[..],
        &[
            Color::from_hsv(0.0, 1.0, 1.0),
            Color::from_hsv(60.0, 1.0, 1.0),
            Color::from_hsv(154.8235294117647, 0.6901960784313725, 1.0),
            Color::from_hsv(251.76470588235293, 0.7529411764705882, 0.6274509803921569),
            Color::from_hsv(126.11764705882354, 0.7137254901960784, 1.0),
            Color::from_hsv(179.05882352941177, 0.615686274509804, 0.8392156862745098),
        ][..]
    );
    assert_eq!(*effect.speed, 65);
    assert_eq!(
        effect.source_control_speed,
        Some(SourceControl {
            input_min: 24,
            input_max: 100,
            output_min: 27,
            output_max: 100,
        })
    );
    assert_eq!(effect.source_control_brightness, None);
    assert!(effect.fade);
    assert!(!effect.random_color);
    assert!(!effect.slide_colors);

    // Controller 4 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 45);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Sequence(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.background, Color::from_hsv(0.0, 0.0, 0.0));
    assert_eq!(
        &effect.colors[..],
        &[Color::from_rgb_hex(0xFF0000), Color::from_rgb_hex(0x0000FF)][..]
    );
    assert_eq!(*effect.speed, 40);
    assert_eq!(*effect.smoothness, 25);
    assert_eq!(*effect.delay_after_sequence, 19);
    assert_eq!(*effect.delay_before_sequence, 15);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(effect.reverse_direction);
    assert!(!effect.fade);
    assert!(!effect.random_color);

    // Controller 5 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 60);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Laser(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.background, Color::from_rgb_hex(0x0F0F0F));
    assert_eq!(effect.inner_color, Color::from_rgb_hex(0x0000FF));
    assert_eq!(effect.outer_color, Color::from_rgb_hex(0x00FF00));
    assert_eq!(*effect.speed, 22);
    assert_eq!(*effect.smoothness, 49);
    assert_eq!(*effect.width, 16);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.reverse_direction);
    assert!(!effect.fade);
    assert!(!effect.random_color);
    assert!(!effect.second_color_mode);
    assert!(effect.color_change);
    assert!(effect.circular);

    // Controller 6 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 75);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, Some(DataSource::Flow));
    assert_eq!(controller.sensor_attenuation_rising, 29);
    assert_eq!(controller.sensor_attenuation_falling, 23);
    let Effect::ColorSequence(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        &effect.colors[..],
        &[
            Color::from_rgb_hex(0xFF0000),
            Color::from_rgb_hex(0xFFFF00),
            Color::from_rgb_hex(0x00FF00),
            Color::from_rgb_hex(0x00FFFF),
            Color::from_rgb_hex(0x0000FF),
            Color::from_rgb_hex(0xFF00FF)
        ][..]
    );
    assert_eq!(*effect.speed, 11);
    assert_eq!(*effect.smoothness, 49);
    assert_eq!(*effect.color_change_speed, 67);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(
        effect.source_control_brightness,
        Some(SourceControl {
            input_min: 12,
            input_max: 345,
            output_min: 31,
            output_max: 217,
        })
    );
    assert!(effect.reverse_direction);
    assert!(!effect.random_color);

    assert!(strip.next().is_none());

    // Controller 7 (Sensor)
    let controller = sensor.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 10);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::ColorShift(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.color,
        Color::from_hsv(115.05882352941177, 0.7137254901960784, 1.0)
    );
    assert_eq!(*effect.speed, 24);
    assert_eq!(*effect.color_range, 57);
    assert_eq!(*effect.total_area, 47);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.reverse_direction);

    // Controller 8 (Sensor)
    let controller = sensor.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 10);
    assert_eq!(controller.data_source, Some(DataSource::WaterTemperature));
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::BarGraph(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.background, Color::from_rgb_hex(0x000000));
    assert_eq!(effect.peak_color, Color::from_rgb_hex(0xFFFFFF));
    assert_eq!(
        &effect.colors[..],
        &[
            (Color::from_hsv(240.0, 1.0, 1.0), 30, false),
            (Color::from_hsv(120.0, 1.0, 1.0), 40, false),
            (Color::from_hsv(60.0, 1.0, 1.0), 50, false),
            (Color::from_hsv(0.0, 1.0, 1.0), 60, true),
        ][..]
    );
    assert_eq!(effect.end_value, 70);
    assert_eq!(*effect.rotation, 0);
    assert_eq!(*effect.peak_hold_time, 22);
    assert_eq!(
        effect.source_control_rotation,
        Some(SourceControl {
            input_min: 20,
            input_max: 70,
            output_min: 0,
            output_max: 100
        })
    );
    assert!(!effect.reverse_direction);
    assert!(effect.show_peak);
    assert!(effect.show_bar);
    assert!(!effect.show_ranges);
    assert!(effect.fade_ranges);

    assert!(sensor.next().is_none());
}

#[test]
fn effects_1() {
    let mut reader = File::open("tests/assets/effects_1.frame").unwrap();
    let frame = Frame::decode(&mut reader).unwrap();

    let Frame::Settings(values) = frame;

    macro_rules! flow_correction {
        ($flow:expr, $value:expr) => {
            (
                Flow::from_value($flow).unwrap(),
                FlowCorrection::from_value($value).unwrap(),
            )
        };
    }

    macro_rules! chart {
        ($source:ident, $interval:expr) => {
            Chart {
                source: ChartSource::$source,
                interval: ChartInterval::from_value($interval).unwrap(),
            }
        };
    }

    /* System */

    assert_eq!(
        values.system.standby_flags,
        StandbyFlags::DISABLE_ALARM_DETECT
            | StandbyFlags::DISPLAY_OFF
            | StandbyFlags::LEDS_DISABLED
            | StandbyFlags::DISABLE_VOLUME_COUNTER
    );
    assert_eq!(*values.system.aqua_bus_address, 58);
    assert_eq!(values.system.increased_current_draw.as_deref(), Some(&600));

    /* Sensor */

    assert_eq!(values.sensor.medium, Medium::DistilledWater);
    assert_eq!(
        values.sensor.connector_type,
        ConnectorType::InnerDiameterLt7mm
    );
    assert_eq!(
        values.sensor.flow_correction,
        [
            flow_correction!(200, 1000),
            flow_correction!(300, -1000),
            flow_correction!(500, 500),
            flow_correction!(700, -500),
            flow_correction!(1000, 1500),
            flow_correction!(1250, -1523),
            flow_correction!(1500, 2512),
            flow_correction!(2000, -2579),
            flow_correction!(2500, 3033),
            flow_correction!(3000, -3333)
        ]
    );

    assert_eq!(*values.sensor.water_temp_offset, -51);
    assert_eq!(*values.sensor.external_temp_offset, 1055);

    assert_eq!(*values.sensor.conductivity_offset, 123);
    assert_eq!(*values.sensor.water_quality_max, 453);
    assert_eq!(*values.sensor.water_quality_min, 963);

    assert_eq!(
        values.sensor.power_flags,
        PowerFlags::AUTOMATIC_POWER_OFFSET_COMPENSATION
    );
    assert_eq!(*values.sensor.power_damping, 616);

    /* Alarms */

    assert_eq!(
        values.alarms.flags,
        AlarmFlags::ENABLE_ACUSTIC_INDICATOR | AlarmFlags::ENABLE_OPTICAL_INDICATOR
    );
    assert_eq!(*values.alarms.startup_delay, 10);
    assert_eq!(values.alarms.flow_alarm_limit, None);
    assert_eq!(
        values.alarms.water_temperature_limit.as_deref(),
        Some(&4510)
    );
    assert_eq!(
        values.alarms.external_temperature_limit.as_deref(),
        Some(&5680)
    );
    assert_eq!(values.alarms.water_quality_limit.as_deref(), Some(&3329));
    assert_eq!(values.alarms.output_signal, OutputSignal::PermanentOn);

    /* Display Settings */

    assert_eq!(values.display.temperature_unit, TemperatureUnit::F);
    assert_eq!(values.display.flow_unit, FlowUnit::Liter);
    assert_eq!(
        values.display.display_flags,
        DisplayFlags::ROTATE | DisplayFlags::DISABLE_BUTTONS
    );
    assert_eq!(values.display.next_page_interval, None);
    assert_eq!(
        values.display.page_flags,
        PageFlags::FLOW_WATERTEMP
            | PageFlags::COND_QUALITY
            | PageFlags::TEMPERATURES
            | PageFlags::FLOW_VOLUME
    );
    assert_eq!(
        values.display.display_brightness,
        DisplayBrightness::Maximum
    );
    assert_eq!(values.display.idle_display_brightness, None);
    assert_eq!(
        values.display.charts,
        [
            chart!(SystemVoltage, 1),
            chart!(Conductivity, 50),
            chart!(ExternalTemp, 100),
            chart!(WaterTemp, 600),
        ]
    );

    /* Lighting */

    let Some(lighting) = values.lighting else {
        panic!("Lighting is expected to be enabled!");
    };

    let mut strip = lighting.strip_controllers.into_iter();
    let mut sensor = lighting.sensor_controllers.into_iter();
    assert_eq!(*lighting.brightness, 230);

    // Controller 1 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Flame(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(47.05882352941177, 0.7843137254901961, 0.09803921568627451)
    );
    assert_eq!(
        effect.color_primary,
        Color::from_hsv(47.05882352941177, 1.0, 0.5882352941176471)
    );
    assert_eq!(
        effect.color_secondary,
        Color::from_hsv(28.235294117647058, 1.0, 1.0)
    );
    assert_eq!(*effect.intensity, 50);
    assert_eq!(effect.source_control_intensity, None);

    // Controller 2 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 15);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Rain(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(0.0, 0.0, 0.00392156862745098)
    );
    assert_eq!(effect.color, Color::from_hsv(28.235294117647058, 0.0, 1.0));
    assert_eq!(*effect.speed, 26);
    assert_eq!(*effect.items, 3);
    assert_eq!(*effect.size, 59);
    assert_eq!(*effect.smoothness, 17);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(effect.reverse_direction);
    assert!(effect.random_color);

    // Controller 3 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 30);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Snow(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(0.0, 0.0, 0.00392156862745098)
    );
    assert_eq!(effect.color, Color::from_hsv(28.235294117647058, 0.0, 1.0));
    assert_eq!(*effect.speed, 28);
    assert_eq!(*effect.items, 4);
    assert_eq!(*effect.size, 22);
    assert_eq!(*effect.smoothness, 35);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.reverse_direction);
    assert!(!effect.random_color);

    // Controller 4 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 45);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Stardust(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(0.0, 0.0, 0.00392156862745098)
    );
    assert_eq!(effect.color, Color::from_hsv(28.235294117647058, 0.0, 1.0));
    assert_eq!(*effect.speed, 71);
    assert_eq!(*effect.items, 4);
    assert_eq!(*effect.size, 38);
    assert_eq!(*effect.smoothness, 76);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.reverse_direction);
    assert!(effect.random_color);

    // Controller 5 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 60);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, Some(DataSource::Flow));
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::ColorSwitch(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        &effect.colors[..],
        &[
            (Color::from_rgb_hex(0xFF0000), 10, true),
            (Color::from_rgb_hex(0xFF0000), 20, false),
            (Color::from_rgb_hex(0xFFFF00), 30, false),
            (Color::from_rgb_hex(0x00FF00), 40, false),
            (Color::from_rgb_hex(0x0000FF), 50, false),
            (Color::from_rgb_hex(0xFFFFFF), 60, false),
        ][..]
    );
    assert_eq!(effect.end_value, 70);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.fade_ranges);

    // Controller 6 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 75);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::SwipingRainbow(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.point_color, Color::from_rgb_hex(0xFFFFFF));
    assert_eq!(effect.strip_color, Color::from_rgb_hex(0xFFFF00));
    assert_eq!(*effect.point_speed, 100);
    assert_eq!(*effect.point_smoothness, 17);
    assert_eq!(*effect.point_size, 8);
    assert_eq!(*effect.color_change_speed, 66);
    assert_eq!(*effect.color_range, 31);
    assert_eq!(effect.source_control_speed, None);
    assert_eq!(effect.source_control_brightness, None);
    assert!(!effect.reverse_direction);

    assert!(strip.next().is_none());

    // Controller 7 (Sensor)
    let controller = sensor.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 10);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::SoundFlash(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(30.11764705882353, 1.0, 0.0784313725490196)
    );
    assert_eq!(
        effect.colors,
        [
            Color::from_rgb_hex(0x0000FF),
            Color::from_rgb_hex(0xFF0000),
            Color::from_rgb_hex(0x00FF00),
            Color::from_rgb_hex(0xFF00FF)
        ]
    );

    // Controller 8 (Sensor)
    let controller = sensor.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 10);
    assert_eq!(controller.data_source, Some(DataSource::Sound));
    assert_eq!(controller.sensor_attenuation_rising, 1);
    assert_eq!(controller.sensor_attenuation_falling, 1);
    let Effect::SoundBars(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(0.0, 0.0, 0.0196078431372549)
    );
    assert_eq!(effect.peak_color, Color::from_rgb_hex(0xFFFFFF));
    assert_eq!(
        &effect.colors[..],
        &[
            (Color::from_hsv(0.0, 1.0, 1.0), 0, false),
            (Color::from_hsv(60.0, 1.0, 1.0), 25, false),
        ][..]
    );
    assert_eq!(effect.end_value, 100);
    assert_eq!(*effect.rotation, 0);
    assert_eq!(*effect.peak_hold_time, 5);
    assert_eq!(effect.source_control_rotation, None);
    assert!(!effect.reverse_direction);
    assert!(!effect.show_peak);
    assert!(effect.show_bar);
    assert!(effect.show_ranges);
    assert!(!effect.fade_ranges);

    assert!(sensor.next().is_none());
}

#[test]
fn effects_2() {
    let mut reader = File::open("tests/assets/effects_2.frame").unwrap();
    let frame = Frame::decode(&mut reader).unwrap();

    let Frame::Settings(values) = frame;

    macro_rules! flow_correction {
        ($flow:expr, $value:expr) => {
            (
                Flow::from_value($flow).unwrap(),
                FlowCorrection::from_value($value).unwrap(),
            )
        };
    }

    macro_rules! chart {
        ($source:ident, $interval:expr) => {
            Chart {
                source: ChartSource::$source,
                interval: ChartInterval::from_value($interval).unwrap(),
            }
        };
    }

    /* System */

    assert_eq!(
        values.system.standby_flags,
        StandbyFlags::DISABLE_ALARM_DETECT
            | StandbyFlags::DISPLAY_OFF
            | StandbyFlags::LEDS_DISABLED
            | StandbyFlags::DISABLE_VOLUME_COUNTER
    );
    assert_eq!(*values.system.aqua_bus_address, 58);
    assert_eq!(values.system.increased_current_draw.as_deref(), Some(&600));

    /* Sensor */

    assert_eq!(values.sensor.medium, Medium::DistilledWater);
    assert_eq!(
        values.sensor.connector_type,
        ConnectorType::InnerDiameterLt7mm
    );
    assert_eq!(
        values.sensor.flow_correction,
        [
            flow_correction!(200, 1000),
            flow_correction!(300, -1000),
            flow_correction!(500, 500),
            flow_correction!(700, -500),
            flow_correction!(1000, 1500),
            flow_correction!(1250, -1523),
            flow_correction!(1500, 2512),
            flow_correction!(2000, -2579),
            flow_correction!(2500, 3033),
            flow_correction!(3000, -3333)
        ]
    );

    assert_eq!(*values.sensor.water_temp_offset, -51);
    assert_eq!(*values.sensor.external_temp_offset, 1055);

    assert_eq!(*values.sensor.conductivity_offset, 123);
    assert_eq!(*values.sensor.water_quality_max, 453);
    assert_eq!(*values.sensor.water_quality_min, 963);

    assert_eq!(
        values.sensor.power_flags,
        PowerFlags::AUTOMATIC_POWER_OFFSET_COMPENSATION
    );
    assert_eq!(*values.sensor.power_damping, 616);

    /* Alarms */

    assert_eq!(
        values.alarms.flags,
        AlarmFlags::ENABLE_ACUSTIC_INDICATOR | AlarmFlags::ENABLE_OPTICAL_INDICATOR
    );
    assert_eq!(*values.alarms.startup_delay, 10);
    assert_eq!(values.alarms.flow_alarm_limit, None);
    assert_eq!(
        values.alarms.water_temperature_limit.as_deref(),
        Some(&4510)
    );
    assert_eq!(
        values.alarms.external_temperature_limit.as_deref(),
        Some(&5680)
    );
    assert_eq!(values.alarms.water_quality_limit.as_deref(), Some(&3329));
    assert_eq!(values.alarms.output_signal, OutputSignal::PermanentOn);

    /* Display Settings */

    assert_eq!(values.display.temperature_unit, TemperatureUnit::F);
    assert_eq!(values.display.flow_unit, FlowUnit::Liter);
    assert_eq!(
        values.display.display_flags,
        DisplayFlags::ROTATE | DisplayFlags::DISABLE_BUTTONS
    );
    assert_eq!(values.display.next_page_interval, None);
    assert_eq!(
        values.display.page_flags,
        PageFlags::FLOW_WATERTEMP
            | PageFlags::COND_QUALITY
            | PageFlags::TEMPERATURES
            | PageFlags::FLOW_VOLUME
    );
    assert_eq!(
        values.display.display_brightness,
        DisplayBrightness::Maximum
    );
    assert_eq!(values.display.idle_display_brightness, None);
    assert_eq!(
        values.display.charts,
        [
            chart!(SystemVoltage, 1),
            chart!(Conductivity, 50),
            chart!(ExternalTemp, 100),
            chart!(WaterTemp, 600),
        ]
    );

    /* Lighting */

    let Some(lighting) = values.lighting else {
        panic!("Lighting is expected to be enabled!");
    };

    let mut strip = lighting.strip_controllers.into_iter();
    let mut sensor = lighting.sensor_controllers.into_iter();
    assert_eq!(*lighting.brightness, 230);

    // Controller 1 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 0);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::SoundSlider(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.background, Color::from_rgb_hex(0x000000));
    assert_eq!(
        effect.effects,
        [
            (
                Color::from_rgb_hex(0x0000FF),
                SoundEffect::InwardsToCenterA,
                SoundEffectSpeed::from_value(4).unwrap()
            ),
            (
                Color::from_rgb_hex(0xFF0000),
                SoundEffect::AllLEDs,
                SoundEffectSpeed::from_value(8).unwrap()
            ),
            (
                Color::from_rgb_hex(0x00FF00),
                SoundEffect::InwardsToCenterB,
                SoundEffectSpeed::from_value(4).unwrap()
            ),
            (
                Color::from_rgb_hex(0xFF00FF),
                SoundEffect::FromLeft,
                SoundEffectSpeed::from_value(6).unwrap()
            ),
        ]
    );
    assert_eq!(*effect.rotate_color, 48);

    // Controller 2 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 15);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::SoundShift(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(60.0, 1.0, 0.19607843137254902)
    );
    assert_eq!(
        effect.effects,
        [
            (
                Color::from_rgb_hex(0x0000FF),
                SoundEffectSpeed::from_value(2).unwrap(),
                true
            ),
            (
                Color::from_rgb_hex(0xFF0000),
                SoundEffectSpeed::from_value(5).unwrap(),
                true
            ),
        ]
    );
    assert_eq!(*effect.rotate_color, 22);
    assert_eq!(*effect.idle_speed, 10);
    assert_eq!(*effect.activity_speed, 50);
    assert!(!effect.reverse_direction);

    // Controller 3 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 30);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::Ambient(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(
        effect.background,
        Color::from_hsv(60.0, 1.0, 0.11764705882352941)
    );

    // Controller 4 (Strip)
    let controller = strip.next().unwrap();
    assert_eq!(controller.offset, 45);
    assert_eq!(controller.length, 15);
    assert_eq!(controller.data_source, None);
    assert_eq!(controller.sensor_attenuation_rising, 10);
    assert_eq!(controller.sensor_attenuation_falling, 15);
    let Effect::ColorGradient(effect) = controller.effect else {
        panic!("Unexpected effect!");
    };
    assert_eq!(effect.start_color, Color::from_rgb_hex(0xFF0000));
    assert_eq!(
        &effect.colors[..],
        &[
            (Color::from_rgb_hex(0x00FF00), 250),
            (Color::from_rgb_hex(0x0000FF), 500),
            (Color::from_hsv(30.11764705882353, 0.0, 1.0), 750),
        ][..]
    );
    assert_eq!(*effect.rotation, 0);
    assert_eq!(effect.source_control_rotation, None);
    assert!(!effect.reverse_direction);
    assert!(!effect.reverse_rotation);

    assert!(strip.next().is_none());
    assert!(sensor.next().is_none());
}
