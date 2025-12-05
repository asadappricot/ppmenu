use anyhow::Result;
use zbus::blocking::{Connection, Proxy};

#[derive(PartialEq, Clone, Copy)]
pub enum PowerProfile {
    PowerSaver,
    Balanced,
    Performance,
}

impl PowerProfile {
    pub fn from_name(name: &str) -> Self {
        match name {
            "power-saver" => PowerProfile::PowerSaver,
            "balanced" => PowerProfile::Balanced,
            "performance" => PowerProfile::Performance,
            &_ => todo!(),
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            PowerProfile::PowerSaver => "power-saver",
            PowerProfile::Balanced => "balanced",
            PowerProfile::Performance => "performance",
        }
    }

    pub fn entry(&self) -> &str {
        match self {
            PowerProfile::PowerSaver => "󰌪 | Power Saver",
            PowerProfile::Balanced => "󰾅 | Balanced",
            PowerProfile::Performance => "󱐋 | Performance",
        }
    }

    pub fn all() -> [PowerProfile; 3] {
        [
            PowerProfile::PowerSaver,
            PowerProfile::Balanced,
            PowerProfile::Performance,
        ]
    }

    pub fn get_active(connection: &Connection) -> Result<PowerProfile> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.UPower.PowerProfiles",
            "/org/freedesktop/UPower/PowerProfiles",
            "org.freedesktop.UPower.PowerProfiles"
        )?;

        let active_profile: String = proxy.get_property("ActiveProfile")?;

        Ok(Self::from_name(&active_profile))
    }

    pub fn apply(&self, connection: &Connection) -> Result<()> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.UPower.PowerProfiles",
            "/org/freedesktop/UPower/PowerProfiles",
            "org.freedesktop.UPower.PowerProfiles"
        )?;

        proxy.set_property("ActiveProfile", self.name())?;

        Ok(())
    }
}
