// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;

use amplify::confinement::SmallVec;
use strict_types::typelib::LibBuilder;

#[test]
fn reflect() {
    #[derive(Clone, Debug, Default)]
    #[derive(StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = "Test")]
    struct ByteStr(SmallVec<u8>);

    let builder = LibBuilder::new(libname!("Test"), none!()).transpile::<ByteStr>();
    let lib = builder.compile().unwrap();

    assert_eq!(
        lib.to_string(),
        "typelib Test -- lotus_chance_cinema_FPkSaFT19rf5n37WGMMsmyAyYXzcCQLFQ3kNoQ52ZdBN

-- no dependencies

data ByteStr          :: [Byte]
"
    );
}