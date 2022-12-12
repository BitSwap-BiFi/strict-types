// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.

//! DTL stands for "Data type library".

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::io::Write;

use amplify::confinement::Confined;
use amplify::Wrapper;

use crate::ast::{TranslateContext, TranslateError, TyInner};
use crate::{StenType, Translate, Ty, TyId, TypeName, TypeRef};

pub type TypeIndex = BTreeMap<TyId, TypeName>;

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum InlineRef {
    #[from]
    Name(TypeName),

    #[from]
    Inline(Box<Ty<InlineRef>>),
}

impl TypeRef for InlineRef {}

impl Display for InlineRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef::Name(name) => write!(f, "{}", name),
            InlineRef::Inline(ty) if ty.is_compound() => write!(f, "({})", ty),
            InlineRef::Inline(ty) => write!(f, "{}", ty),
        }
    }
}

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref)]
pub struct LibId(blake3::Hash);

impl Ord for LibId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for LibId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Display for LibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let m = mnemonic::to_string(&self.as_bytes()[14..18]);
            write!(f, "urn:ubideco:setl:{}#{}", self.0, m)
        } else {
            write!(f, "urn:ubideco:setl:{}", self.0)
        }
    }
}

pub struct LibHasher(blake3::Hasher);

pub const SETL_ID_TAG: [u8; 32] = [0u8; 32];

impl LibHasher {
    pub fn new() -> LibHasher { LibHasher(blake3::Hasher::new_keyed(&SETL_ID_TAG)) }

    pub fn input(&mut self, id: TyId) {
        self.0.write_all(id.as_bytes()).expect("hashers do  not error")
    }

    pub fn finish(self) -> LibId { LibId(self.0.finalize()) }
}

pub struct TypeLib {
    pub roots: BTreeSet<TyId>,
    pub index: TypeIndex,
    pub types: Confined<BTreeMap<TypeName, Ty<InlineRef>>, 1, { u16::MAX as usize }>,
}

impl TypeLib {
    pub fn id(&self) -> LibId {
        let mut hasher = LibHasher::new();
        for id in self.roots.iter() {
            hasher.input(*id);
        }
        hasher.finish()
    }
}

impl TryFrom<StenType> for TypeLib {
    type Error = TranslateError;

    fn try_from(root: StenType) -> Result<Self, Self::Error> { root.translate(&mut ()) }
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.types {
            writeln!(f, "data {:16} :: {}", name, ty)?;
        }
        Ok(())
    }
}

impl Translate<TypeLib> for StenType {
    type Context = ();
    type Error = TranslateError;

    fn translate(self, _: &mut Self::Context) -> Result<TypeLib, Self::Error> {
        let id = self.ty.id();

        let index = self.build_index()?;

        let mut ctx = TranslateContext::with(index);
        let root = self.ty.translate(&mut ctx)?;

        let mut lib = ctx.build_lib(bset!(id))?;
        let name = lib.index.get(&id).ok_or(TranslateError::UnknownId(id))?;
        if lib.types.insert(name.clone(), root)?.is_some() {
            return Err(TranslateError::DuplicateName(name.clone()));
        }

        Ok(lib)
    }
}

impl StenType {
    pub fn build_index(&self) -> Result<TypeIndex, TranslateError> {
        let mut index = empty!();
        self.index(&mut index).map(|_| index)
    }

    pub fn index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        if self.name.is_empty() {
            return Ok(());
        }

        let id = self.ty.id();
        let name = TypeName::from(self.name);
        match index.get(&id) {
            None => index.insert(id, name),
            Some(n) if n != &name => return Err(TranslateError::DuplicateName(name)),
            _ => None,
        };

        self.ty.build_index(index)?;

        Ok(())
    }
}

impl Ty<StenType> {
    pub fn build_index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        match self.as_inner() {
            TyInner::Union(fields) => {
                for ty in fields.values() {
                    ty.index(index)?;
                }
            }
            TyInner::Struct(fields) => {
                for ty in fields.values() {
                    ty.index(index)?;
                }
            }
            TyInner::Array(ty, _)
            | TyInner::List(ty, _)
            | TyInner::Set(ty, _)
            | TyInner::Map(_, ty, _) => ty.index(index)?,
            _ => {}
        }
        Ok(())
    }
}