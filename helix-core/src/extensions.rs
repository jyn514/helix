#[cfg(feature = "steel")]
pub mod steel_implementations {

    use std::borrow::Cow;

    use smallvec::SmallVec;
    use steel::{
        rvals::{as_underlying_type, Custom, SteelString},
        steel_vm::{builtin::BuiltInModule, register_fn::RegisterFn},
        SteelVal,
    };

    use helix_stdx::rope::RopeSliceExt;

    use crate::syntax::{AutoPairConfig, SoftWrap};

    impl steel::rvals::Custom for crate::Position {}
    impl steel::rvals::Custom for crate::Selection {}
    impl steel::rvals::Custom for AutoPairConfig {}
    impl steel::rvals::Custom for SoftWrap {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SliceKind {
        Normal(usize, usize),
        Byte(usize, usize),
        Line(usize),
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct SteelRopeSlice {
        text: crate::Rope,
        ranges: SmallVec<[SliceKind; 5]>,
    }

    impl Custom for SteelRopeSlice {
        // `equal?` on two ropes should return true if they are the same
        fn equality_hint(&self, other: &dyn steel::rvals::CustomType) -> bool {
            if let Some(other) = as_underlying_type::<SteelRopeSlice>(other) {
                self == other
            } else {
                false
            }
        }

        fn equality_hint_general(&self, other: &steel::SteelVal) -> bool {
            match other {
                SteelVal::StringV(s) => self.to_slice() == s.as_str(),
                SteelVal::Custom(c) => Self::equality_hint(&self, c.borrow().as_ref()),

                _ => false,
            }
        }
    }

    impl SteelRopeSlice {
        pub fn from_string(string: SteelString) -> Self {
            Self {
                text: crate::Rope::from_str(string.as_str()),
                ranges: SmallVec::default(),
            }
        }

        pub fn new(rope: crate::Rope) -> Self {
            Self {
                text: rope,
                ranges: SmallVec::default(),
            }
        }

        fn to_slice(&self) -> crate::RopeSlice<'_> {
            let mut slice = self.text.slice(..);

            for range in &self.ranges {
                match range {
                    SliceKind::Normal(l, r) => slice = slice.slice(l..r),
                    SliceKind::Byte(l, r) => slice = slice.byte_slice(l..r),
                    SliceKind::Line(index) => slice = slice.line(*index),
                }
            }

            slice
        }

        pub fn slice(mut self, lower: usize, upper: usize) -> Self {
            self.ranges.push(SliceKind::Normal(lower, upper));
            self
        }

        pub fn char_to_byte(&self, pos: usize) -> usize {
            self.to_slice().char_to_byte(pos)
        }

        pub fn byte_slice(mut self, lower: usize, upper: usize) -> Self {
            self.ranges.push(SliceKind::Byte(lower, upper));
            self
        }

        pub fn line(mut self, cursor: usize) -> Self {
            self.ranges.push(SliceKind::Line(cursor));
            self
        }

        pub fn to_string(&self) -> String {
            self.to_slice().to_string()
        }

        pub fn len_chars(&self) -> usize {
            self.to_slice().len_chars()
        }

        pub fn get_char(&self, index: usize) -> Option<char> {
            self.to_slice().get_char(index)
        }

        pub fn len_lines(&self) -> usize {
            self.to_slice().len_lines()
        }

        pub fn trim_start(mut self) -> Self {
            for (idx, c) in self.to_slice().chars().enumerate() {
                if !c.is_whitespace() {
                    self.ranges.push(SliceKind::Normal(0, idx));
                    break;
                }
            }

            self
        }

        pub fn trimmed_starts_with(&self, pat: SteelString) -> bool {
            let maybe_owned = Cow::from(self.to_slice());

            maybe_owned.trim_start().starts_with(pat.as_str())
        }

        pub fn starts_with(&self, pat: SteelString) -> bool {
            self.to_slice().starts_with(pat.as_str())
        }

        pub fn ends_with(&self, pat: SteelString) -> bool {
            self.to_slice().ends_with(pat.as_str())
        }
    }

    pub fn rope_module() -> BuiltInModule {
        let mut module = BuiltInModule::new("helix/core/text");

        module
            .register_fn("string->rope", SteelRopeSlice::from_string)
            .register_fn("rope->slice", SteelRopeSlice::slice)
            .register_fn("rope-char->byte", SteelRopeSlice::char_to_byte)
            .register_fn("rope->byte-slice", SteelRopeSlice::byte_slice)
            .register_fn("rope->line", SteelRopeSlice::line)
            .register_fn("rope->string", SteelRopeSlice::to_string)
            .register_fn("rope-len-chars", SteelRopeSlice::len_chars)
            .register_fn("rope-char-ref", SteelRopeSlice::get_char)
            .register_fn("rope-len-lines", SteelRopeSlice::len_lines)
            .register_fn("rope-starts-with?", SteelRopeSlice::starts_with)
            .register_fn("rope-ends-with?", SteelRopeSlice::ends_with)
            .register_fn("rope-trim-start", SteelRopeSlice::trim_start);

        module
    }
}
