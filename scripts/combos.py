"""
Generates insim_core/src/track.rs

I was too lazy to try and write this as a macro or as a build.rs file.
Sorry.
"""

from typing import NamedTuple, Optional, List
from enum import Enum

from jinja2 import Environment, BaseLoader

rtemplate = Environment(loader=BaseLoader()).from_string("""
use crate::license::License;
use binrw::{BinRead, BinWrite};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Handles parsing a Track name.
#[non_exhaustive]
pub enum Track {
    {%- for key, info in data.items() %}
    {%- if loop.first %}
    #[default]
    {%- endif %}
    {{ key }},
    {%- endfor %}
}

impl Track {

    fn license(&self) -> License {
        match self {
            {%- for key, info in data.items() %}
            Self::{{ key }} => License::{{ info.license.value }},
            {%- endfor %}
        }
    }

    pub fn distance_km(&self) -> Option<f32> {
        self.distance_mile().map(|distance| distance * 1.60934)
    }

    pub fn distance_mile(&self) -> Option<f32> {
        match self {
            {%- for key, info in data.items() %}
            Self::{{ key }} => {% if not info.distance %}None{% else %}Some({{ info.distance }}){% endif %},
            {%- endfor %}
        }
    }

    // Complete name
    pub fn complete_name(&self) -> String {
        match self {
            {%- for key, info in data.items() %}
            Self::{{ key }} => "{{ info.name }}",
            {%- endfor %}
        }.to_string()

    }

    /// Code
    pub fn code(&self) -> String {
        match self {
            {%- for key, info in data.items() %}
            Self::{{ key }} => "{{ info.id.upper() }}",
            {%- endfor %}
        }.to_string()
    }

    /// Is this a reversed track?
    pub fn is_reverse(&self) -> bool {
        matches!(self,
            {%- for key, info in data.items() %}
            {%- if info.reverse %}
            {%- if not loop.first %} | {% endif %} Self::{{ key }}
            {%- endif %}
            {%- endfor %}
        )
    }

    /// Is this an open world track?
    pub fn is_open(&self) -> bool {
        matches!(self,
            {%- for key, info in data.items() %}
            {%- if info.open %}
            {%- if not loop.first %} | {% endif %} Self::{{ key }}
            {%- endif %}
            {%- endfor %}
        )
    }

}

impl BinRead for Track {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let pos = reader.stream_position()?;

        <[u8; 6]>::read_options(reader, endian, args).map(|bytes| match bytes {
            {%- for key, info in data.items() %}
            {{ info.raw }} => Ok(Self::{{ key }}),
            {%- endfor %}
            _ => {
                Err(binrw::Error::NoVariantMatch { pos })
            },
        })?
    }
}


impl BinWrite for Track {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            {%- for key, info in data.items() %}
            Self::{{ key }} => {{ info.raw }}.write_options(writer, endian, args),
            {%- endfor %}
        }
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}
""")

try:
    from typing import Self
except ImportError:
    from typing_extensions import Self


class License(Enum):
    DEMO = 'Demo'
    S1 = 'S1'
    S2 = 'S2'
    S3 = 'S3'


class Location(NamedTuple):
    prefix: str
    name: str
    license: License


class Configuration(NamedTuple):
    suffix: Optional[str]
    distance: Optional[float]
    open: bool
    reverse: bool

    @classmethod
    def all(cls, distance=None) -> List[Self]:
        return [
            Configuration(suffix="", distance=distance, open=False, reverse=False),
            Configuration(suffix="R", distance=distance, open=False, reverse=True),
            Configuration(suffix="X", distance=None, open=True, reverse=False),
            Configuration(suffix="Y", distance=None, open=True, reverse=True),
        ]

    @classmethod
    def normal_x(cls, distance=None) -> List[Self]:
        return [
            Configuration(suffix="", distance=distance, open=False, reverse=False),
            Configuration(suffix="X", distance=None, open=True, reverse=False),
        ]


class Variation(NamedTuple):
    id: int
    name: str
    configs: List[Configuration]


class MaterialisedCombo(NamedTuple):
    id: str
    name: str
    distance: Optional[float]
    license: License
    raw: str
    open: bool
    reverse: bool


COMBOS = {
    Location(prefix="BL", name="Blackwood", license=License.DEMO): [
        Variation(id=1, name="GP Track", configs=Configuration.all(distance=2.0)),
        Variation(id=2, name="Historic", configs=Configuration.all(distance=2.0)),
        Variation(id=3, name="Rallycross", configs=Configuration.all(distance=2.0)),
        Variation(id=4, name="Carpark", configs=Configuration.normal_x()),
    ],

    Location(prefix="SO", name="South City", license=License.S1): [
        Variation(id=1, name="Classic", configs=Configuration.all(distance=1.3)),
        Variation(id=2, name="Sprint 1", configs=Configuration.all(distance=1.3)),
        Variation(id=3, name="Sprint 2", configs=Configuration.all(distance=0.8)),
        Variation(id=4, name="City Long", configs=Configuration.all(distance=2.5)),
        Variation(id=5, name="Town Course", configs=Configuration.all(distance=2.0)),
        Variation(id=6, name="Chicane Course", configs=Configuration.all(distance=1.8)),
    ],

    Location(prefix="FE", name="Fern Bay", license=License.S1): [
        Variation(id=1, name="Club", configs=Configuration.all(distance=1.0)),
        Variation(id=2, name="Green", configs=Configuration.all(distance=1.9)),
        Variation(id=3, name="Gold", configs=Configuration.all(distance=2.2)),
        Variation(id=4, name="Black", configs=Configuration.all(distance=4.1)),
        Variation(id=5, name="Rallycross", configs=Configuration.all(distance=1.3)),
        Variation(id=6, name="Rallycross Green", configs=Configuration.all(distance=0.5)),
    ],

    Location(prefix="AU", name="", license=License.S1): [
        Variation(id=1, name="Autocross", configs=Configuration.normal_x()),
        Variation(id=2, name="Skid Pad", configs=Configuration.normal_x()),
        Variation(id=3, name="Drag Strip", configs=Configuration.normal_x()),
        Variation(id=4, name="8 Lane Drag Strip", configs=Configuration.normal_x()),
    ],

    Location(prefix="KY", name="Kyoto", license=License.S2): [
        Variation(id=1, name="Oval", configs=Configuration.all(distance=1.9)),
        Variation(id=2, name="National", configs=Configuration.all(distance=3.2)),
        Variation(id=3, name="GP Long", configs=Configuration.all(distance=4.6)),
    ],

    Location(prefix="WE", name="Westhill", license=License.S2): [
        Variation(id=1, name="National", configs=Configuration.all(distance=2.7)),
        Variation(id=2, name="International", configs=Configuration.all(distance=3.6)),
        Variation(id=3, name="Car Park", configs=Configuration.normal_x()),
        Variation(id=4, name="Karting", configs=Configuration.all(distance=0.3)),
        Variation(id=5, name="Karting Long", configs=Configuration.all(distance=0.8)),
    ],

    Location(prefix="AS", name="Aston", license=License.S2): [
        Variation(id=1, name="Cadet", configs=Configuration.all(distance=1.2)),
        Variation(id=2, name="Club", configs=Configuration.all(distance=1.9)),
        Variation(id=3, name="National", configs=Configuration.all(distance=3.5)),
        Variation(id=4, name="Historic", configs=Configuration.all(distance=5.0)),
        Variation(id=5, name="Grand Prix", configs=Configuration.all(distance=5.5)),
        Variation(id=6, name="Grand Touring", configs=Configuration.all(distance=5.0)),
        Variation(id=7, name="North", configs=Configuration.all(distance=3.2)),
    ],

    Location(prefix="RO", name="Rockingham", license=License.S3): [
        Variation(id=1, name="ISSC", configs=Configuration.normal_x(distance=1.9)),
        Variation(id=2, name="National", configs=Configuration.normal_x(distance=1.7)),
        Variation(id=3, name="Oval", configs=Configuration.normal_x(distance=1.5)),
        Variation(id=4, name="ISSC Long", configs=Configuration.normal_x(distance=2.0)),
        Variation(id=5, name="Lake", configs=Configuration.normal_x(distance=0.7)),
        Variation(id=6, name="Handling", configs=Configuration.normal_x(distance=1.0)),
        Variation(id=7, name="International", configs=Configuration.normal_x(distance=2.4)),
        Variation(id=8, name="Historic", configs=Configuration.normal_x(distance=2.2)),
        Variation(id=9, name="Historic Short", configs=Configuration.normal_x(distance=1.4)),
        Variation(id=10, name="International Long", configs=Configuration.normal_x(distance=2.5)),
        Variation(id=11, name="Sportscar", configs=Configuration.normal_x(distance=1.7)),
    ],

    Location(prefix="LA", name="Layout Square", license=License.S3): [
        Variation(id=1, name="Long Grid", configs=Configuration.normal_x()),
        Variation(id=2, name="Wide Grid", configs=Configuration.normal_x()),
    ],
}

def main():

    materialised_dict = {}

    for (location_short, location_long, license), combos in COMBOS.items():
        for combo in combos:
            for config in combo.configs:
                key = f"{location_short}{combo.id}{config.suffix}".capitalize()
                name = " ".join(filter(None, [
                    location_long, combo.name, config.suffix or ""
                ]))

                raw = [
                    f"b'{i}'" for i in list(key.upper())
                ] + ["0"] * 6

                raw = ", ".join(raw[:6])

                materialised_dict[key] = MaterialisedCombo(
                    key,
                    name,
                    config.distance,
                    license,
                    f"[{raw}]",
                    config.open,
                    config.reverse
                )

    data = rtemplate.render(data=materialised_dict)
    print(data)


if __name__ == "__main__":
    main()
