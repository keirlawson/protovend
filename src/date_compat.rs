/*
 * Copyright 2020 Skyscanner Limited.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
*/

use chrono::NaiveDateTime;
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt::{self, Formatter};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f";

pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&dt.format(DATE_FORMAT).to_string())
}

pub fn deserialize<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(d.deserialize_str(DateCompatVisitor)?)
}

struct DateCompatVisitor;

impl<'de> Visitor<'de> for DateCompatVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "a timestamp in the default Python format")
    }

    fn visit_str<E>(self, value: &str) -> Result<NaiveDateTime, E>
    where
        E: de::Error,
    {
        NaiveDateTime::parse_from_str(value, DATE_FORMAT)
            .map_err(|_| E::custom(format!("value cannot be parsed: {}", value)))
    }
}
