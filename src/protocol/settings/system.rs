use bitflags::bitflags;

use crate::misc::{Decode, Guard, GuardOutput, IoError, Reader};
use crate::{define_wrapped, impl_ranged};

/// System related settings for a high flow NEXT device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SystemSettings {
    /// Stand-by flags.
    pub standby_flags: StandbyFlags,

    /// Aqua-Bus address assigned to this device.
    pub aqua_bus_address: AquaBusAddress,

    /// Whether to enable increased USB current draw or not.
    pub increased_current_draw: Option<CurrentDraw>,
}

bitflags! {
    /// Stand-by flags used in [`SystemSettings::standby_flags`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct StandbyFlags: u8 {
        /// Enter standby if USB is not connected.
        const STANDBY_NO_USB = 0x01;

        /// Enter standby upon USB suspend command.
        const STANDBY_ON_SUSPEND = 0x02;

        /// Enter standby if aquabus connection is lost.
        const STANDBY_ON_ABUS_LOSS = 0x04;

        /// Disable alarm detection in standby.
        const DISABLE_ALARM_DETECT = 0x10;

        /// Disable display in standby.
        const DISPLAY_OFF = 0x20;

        /// Disable LEDs in standby.
        const LEDS_DISABLED = 0x40;

        /// Disable volume counter in standby.
        const DISABLE_VOLUME_COUNTER = 0x80;
    }
}

impl Decode for StandbyFlags {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let bits = reader.read_u8()?;

        Ok(R::guard(|_| Self::from_bits_truncate(bits)))
    }
}

define_wrapped! {
    /// Aqua-Bus address assigned to a device used in [`SystemSettings::aqua_bus_address`].
    ///
    /// Valid value range: 58..61
    pub type AquaBusAddress<u8, AquaBusAddressTag>;
}
impl_ranged!(AquaBusAddress<u8, AquaBusAddressTag>, 58, 61);

define_wrapped! {
    /// Increased USB current draw used in [`SystemSettings::increased_current_draw`].
    ///
    /// Valid value range: 500..2000 in milli ampere (mA)
    pub type CurrentDraw<u16, CurrentDrawTag>;
}
impl_ranged!(CurrentDraw<u16, CurrentDrawTag>, 500, 2000);

impl Decode for Option<CurrentDraw> {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        reader.skip::<1>()?;
        let flags = reader.read_u8()?;
        let val = reader.read_u16be()?;

        let ret = R::guard(|_| {
            if flags & 0x01 != 0 {
                Ok(Some(CurrentDraw::from_value(val)?))
            } else {
                Ok(None)
            }
        });

        R::Guard::transpose_result(ret)
    }
}
