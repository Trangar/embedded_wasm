use super::MemArg;
use crate::{
    instruction::{
        LaneIdx,
        Signedness::{Signed, Unsigned},
        VectorInstruction,
        VectorInstruction::*,
    },
    ErrorKind, Mark, ParseResult, Reader,
};

pub fn unknown_instruction<'a>(
    _: &mut Reader<'a>,
    mark: Mark<'a>,
) -> ParseResult<'a, VectorInstruction> {
    Err(mark.into_error(ErrorKind::UnknownVectorInstruction))
}

pub const INSTRUCTIONS: [for<'a> fn(
    &mut Reader<'a>,
    mark: Mark<'a>,
) -> ParseResult<'a, VectorInstruction>; 256] = [
    // 0
    |reader, _mark| Ok(V128Load(MemArg::parse(reader)?)),
    // 1
    |reader, _mark| Ok(V128Load8x8(MemArg::parse(reader)?, Signed)),
    // 2
    |reader, _mark| Ok(V128Load8x8(MemArg::parse(reader)?, Unsigned)),
    // 3
    |reader, _mark| Ok(V128Load16x4(MemArg::parse(reader)?, Signed)),
    // 4
    |reader, _mark| Ok(V128Load16x4(MemArg::parse(reader)?, Unsigned)),
    // 5
    |reader, _mark| Ok(V128Load32x2(MemArg::parse(reader)?, Signed)),
    // 6
    |reader, _mark| Ok(V128Load32x2(MemArg::parse(reader)?, Unsigned)),
    // 7
    |reader, _mark| Ok(V128Load8Splat(MemArg::parse(reader)?)),
    // 8
    |reader, _mark| Ok(V128Load16Splat(MemArg::parse(reader)?)),
    // 9
    |reader, _mark| Ok(V128Load32Splat(MemArg::parse(reader)?)),
    // 10
    |reader, _mark| Ok(V128Load64Splat(MemArg::parse(reader)?)),
    // 11
    |reader, _mark| Ok(V128Store(MemArg::parse(reader)?)),
    // 12
    |reader, _mark| Ok(V128Const(reader.read_exact()?)),
    // 13
    |reader, _mark| {
        let mark = reader.mark();
        let slice: [u8; 16] = reader.read_exact()?;
        let mut lanes = [LaneIdx::default(); 16];
        for (idx, lane) in lanes.iter_mut().enumerate() {
            *lane = LaneIdx(slice[idx]);
            if lane.0 >= 16 {
                return Err(mark.into_error(ErrorKind::InvalidLaneIndex { max: 16 }));
            }
        }
        Ok(I8x16Shuffle(lanes))
    },
    // 14
    |_, _| Ok(I8x16Swizzle),
    // 15
    |_, _| Ok(I8x16Splat),
    // 16
    |_, _| Ok(I16x8Splat),
    // 17
    |_, _| Ok(I32x4Splat),
    // 18
    |_, _| Ok(I64x2Splat),
    // 19
    |_, _| Ok(F32x4Splat),
    // 20
    |_, _| Ok(F64x2Splat),
    // 21
    |reader, _mark| Ok(I8x16ExtractLane(LaneIdx::parse_max_16(reader)?, Signed)),
    // 22
    |reader, _mark| Ok(I8x16ExtractLane(LaneIdx::parse_max_16(reader)?, Unsigned)),
    // 23
    |reader, _mark| Ok(I8x16ReplaceLane(LaneIdx::parse_max_16(reader)?)),
    // 24
    |reader, _mark| Ok(I16x8ExtractLane(LaneIdx::parse_max_8(reader)?, Signed)),
    // 25
    |reader, _mark| Ok(I16x8ExtractLane(LaneIdx::parse_max_8(reader)?, Unsigned)),
    // 26
    |reader, _mark| Ok(I16x8ReplaceLane(LaneIdx::parse_max_8(reader)?)),
    // 27
    |reader, _mark| Ok(I32x4ExtractLane(LaneIdx::parse_max_4(reader)?)),
    // 28
    |reader, _mark| Ok(I32x4ReplaceLane(LaneIdx::parse_max_4(reader)?)),
    // 29
    |reader, _mark| Ok(I64x2ExtractLane(LaneIdx::parse_max_2(reader)?)),
    // 30
    |reader, _mark| Ok(I64x2ReplaceLane(LaneIdx::parse_max_2(reader)?)),
    // 31
    |reader, _mark| Ok(F32x4ExtractLane(LaneIdx::parse_max_4(reader)?)),
    // 32
    |reader, _mark| Ok(F32x4ReplaceLane(LaneIdx::parse_max_4(reader)?)),
    // 33
    |reader, _mark| Ok(F64x2ExtractLane(LaneIdx::parse_max_2(reader)?)),
    // 34
    |reader, _mark| Ok(F64x2ReplaceLane(LaneIdx::parse_max_2(reader)?)),
    // 35
    |_, _| Ok(I8x16Equal),
    // 36
    |_, _| Ok(I8x16NotEqual),
    // 37
    |_, _| Ok(I8x16LessThan(Signed)),
    // 38
    |_, _| Ok(I8x16LessThan(Unsigned)),
    // 39
    |_, _| Ok(I8x16GreaterThan(Signed)),
    // 40
    |_, _| Ok(I8x16GreaterThan(Unsigned)),
    // 41
    |_, _| Ok(I8x16LessOrEqualTo(Signed)),
    // 42
    |_, _| Ok(I8x16LessOrEqualTo(Unsigned)),
    // 43
    |_, _| Ok(I8x16GreaterOrEqualTo(Signed)),
    // 44
    |_, _| Ok(I8x16GreaterOrEqualTo(Unsigned)),
    // 45
    |_, _| Ok(I16x8Equal),
    // 46
    |_, _| Ok(I16x8NotEqual),
    // 47
    |_, _| Ok(I16x8LessThan(Signed)),
    // 48
    |_, _| Ok(I16x8LessThan(Unsigned)),
    // 49
    |_, _| Ok(I16x8GreaterThan(Signed)),
    // 50
    |_, _| Ok(I16x8GreaterThan(Unsigned)),
    // 51
    |_, _| Ok(I16x8LessOrEqualTo(Signed)),
    // 52
    |_, _| Ok(I16x8LessOrEqualTo(Unsigned)),
    // 53
    |_, _| Ok(I16x8GreaterOrEqualTo(Signed)),
    // 54
    |_, _| Ok(I16x8GreaterOrEqualTo(Unsigned)),
    // 55
    |_, _| Ok(I32x4Equal),
    // 56
    |_, _| Ok(I32x4NotEqual),
    // 57
    |_, _| Ok(I32x4LessThan(Signed)),
    // 58
    |_, _| Ok(I32x4LessThan(Unsigned)),
    // 59
    |_, _| Ok(I32x4GreaterThan(Signed)),
    // 60
    |_, _| Ok(I32x4GreaterThan(Unsigned)),
    // 61
    |_, _| Ok(I32x4LessOrEqualTo(Signed)),
    // 62
    |_, _| Ok(I32x4LessOrEqualTo(Unsigned)),
    // 63
    |_, _| Ok(I32x4GreaterOrEqualTo(Signed)),
    // 64
    |_, _| Ok(I32x4GreaterOrEqualTo(Unsigned)),
    // 65
    |_, _| Ok(F32x4Equal),
    // 66
    |_, _| Ok(F32x4NotEqual),
    // 67
    |_, _| Ok(F32x4LessThan),
    // 68
    |_, _| Ok(F32x4GreaterThan),
    // 69
    |_, _| Ok(F32x4LessOrEqualTo),
    // 70
    |_, _| Ok(F32x4GreaterOrEqualTo),
    // 71
    |_, _| Ok(F64x2Equal),
    // 72
    |_, _| Ok(F64x2NotEqual),
    // 73
    |_, _| Ok(F64x2LessThan),
    // 74
    |_, _| Ok(F64x2GreaterThan),
    // 75
    |_, _| Ok(F64x2LessOrEqualTo),
    // 76
    |_, _| Ok(F64x2GreaterOrEqualTo),
    // 77
    |_, _| Ok(V128Not),
    // 78
    |_, _| Ok(V128And),
    // 79
    |_, _| Ok(V128AndNot),
    // 80
    |_, _| Ok(V128Or),
    // 81
    |_, _| Ok(V128Xor),
    // 82
    |_, _| Ok(V128BitSelect),
    // 83
    |_, _| Ok(V128AnyTrue),
    // 84
    |reader, _mark| {
        Ok(V128Load8Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_16(reader)?,
        ))
    },
    // 85
    |reader, _mark| {
        Ok(V128Load16Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_8(reader)?,
        ))
    },
    // 86
    |reader, _mark| {
        Ok(V128Load32Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_4(reader)?,
        ))
    },
    // 87
    |reader, _mark| {
        Ok(V128Load64Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_2(reader)?,
        ))
    },
    // 88
    |reader, _mark| {
        Ok(V128Store8Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_16(reader)?,
        ))
    },
    // 89
    |reader, _mark| {
        Ok(V128Store16Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_8(reader)?,
        ))
    },
    // 90
    |reader, _mark| {
        Ok(V128Store32Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_4(reader)?,
        ))
    },
    // 91
    |reader, _mark| {
        Ok(V128Store64Lane(
            MemArg::parse(reader)?,
            LaneIdx::parse_max_2(reader)?,
        ))
    },
    // 92
    |reader, _mark| Ok(V128Load32Zero(MemArg::parse(reader)?)),
    // 93
    |reader, _mark| Ok(V128Load64Zero(MemArg::parse(reader)?)),
    // 94
    |_, _| Ok(F32x4DemoteF64x2Zero),
    // 95
    |_, _| Ok(F64x2PromoteLowF32x4),
    // 96
    |_, _| Ok(I8x16Abs),
    // 97
    |_, _| Ok(I8x16Neg),
    // 98
    |_, _| Ok(I8x16PopCnt),
    // 99
    |_, _| Ok(I8x16AllTrue),
    // 100
    |_, _| Ok(I8x16Bitmask),
    // 101
    |_, _| Ok(I8x16NarrowI16x8(Signed)),
    // 102
    |_, _| Ok(I8x16NarrowI16x8(Signed)),
    // 103
    |_, _| Ok(F32x4Ceil),
    // 104
    |_, _| Ok(F32x4Floor),
    // 105
    |_, _| Ok(F32x4Trunc),
    // 106
    |_, _| Ok(F32x4Nearest),
    // 107
    |_, _| Ok(I8x16ShiftLeft),
    // 108
    |_, _| Ok(I8x16ShiftRight(Signed)),
    // 109
    |_, _| Ok(I8x16ShiftRight(Unsigned)),
    // 110
    |_, _| Ok(I8x16Add),
    // 111
    |_, _| Ok(I8x16AddSaturating(Signed)),
    // 112
    |_, _| Ok(I8x16AddSaturating(Unsigned)),
    // 113
    |_, _| Ok(I8x16Sub),
    // 114
    |_, _| Ok(I8x16SubSaturating(Signed)),
    // 115
    |_, _| Ok(I8x16SubSaturating(Unsigned)),
    // 116
    |_, _| Ok(F64x2Ceil),
    // 117
    |_, _| Ok(F64x2Floor),
    // 118
    |_, _| Ok(I8x16Min(Signed)),
    // 119
    |_, _| Ok(I8x16Min(Unsigned)),
    // 120
    |_, _| Ok(I8x16Max(Signed)),
    // 121
    |_, _| Ok(I8x16Max(Unsigned)),
    // 122
    |_, _| Ok(F64x2Trunc),
    // 123
    |_, _| Ok(I8x16Average),
    // 124
    |_, _| Ok(I16x8ExtAddPairWiseI8x16(Signed)),
    // 125
    |_, _| Ok(I16x8ExtAddPairWiseI8x16(Unsigned)),
    // 126
    |_, _| Ok(I32x4ExtAddPairwiseI16x8(Signed)),
    // 127
    |_, _| Ok(I32x4ExtAddPairwiseI16x8(Unsigned)),
    // 128
    |_, _| Ok(I16x8Abs),
    // 129
    |_, _| Ok(I16x8Neg),
    // 130
    |_, _| Ok(I16x8Q16MulrSat),
    // 131
    |_, _| Ok(I16x8AllTrue),
    // 132
    |_, _| Ok(I16x8Bitmask),
    // 133
    |_, _| Ok(I16x8NarrowI32x4(Signed)),
    // 134
    |_, _| Ok(I16x8NarrowI32x4(Unsigned)),
    // 135
    |_, _| Ok(I16x8ExtendLowI8x16(Signed)),
    // 136
    |_, _| Ok(I16x8ExtendHighI8x16(Signed)),
    // 137
    |_, _| Ok(I16x8ExtendLowI8x16(Unsigned)),
    // 138
    |_, _| Ok(I16x8ExtendHighI8x16(Unsigned)),
    // 139
    |_, _| Ok(I16x8ShiftLeft),
    // 140
    |_, _| Ok(I16x8ShiftRight(Signed)),
    // 141
    |_, _| Ok(I16x8ShiftRight(Unsigned)),
    // 142
    |_, _| Ok(I16x8Add),
    // 143
    |_, _| Ok(I16x8AddSaturating(Signed)),
    // 144
    |_, _| Ok(I16x8AddSaturating(Unsigned)),
    // 145
    |_, _| Ok(I16x8Sub),
    // 146
    |_, _| Ok(I16x8SubSaturating(Signed)),
    // 147
    |_, _| Ok(I16x8SubSaturating(Unsigned)),
    // 148
    |_, _| Ok(F64x2Nearest),
    // 149
    |_, _| Ok(I16x8Mul),
    // 150
    |_, _| Ok(I16x8Min(Signed)),
    // 151
    |_, _| Ok(I16x8Min(Unsigned)),
    // 152
    |_, _| Ok(I16x8Max(Signed)),
    // 153
    |_, _| Ok(I16x8Max(Unsigned)),
    // 154
    unknown_instruction,
    // 155
    |_, _| Ok(I16x8Average(Unsigned)),
    // 156
    |_, _| Ok(I16x8ExtMulLowI8x16(Signed)),
    // 157
    |_, _| Ok(I16x8ExtMulHighI8x16(Signed)),
    // 158
    |_, _| Ok(I16x8ExtMulLowI8x16(Unsigned)),
    // 159
    |_, _| Ok(I16x8ExtMulHighI8x16(Unsigned)),
    // 160
    |_, _| Ok(I32x4Abs),
    // 161
    |_, _| Ok(I32x4Neg),
    // 162
    unknown_instruction,
    // 163
    |_, _| Ok(I32x4AllTrue),
    // 164
    |_, _| Ok(I32x4Bitmask),
    // 165
    unknown_instruction,
    // 166
    unknown_instruction,
    // 167
    |_, _| Ok(I32x4ExtendLowI16x8(Signed)),
    // 168
    |_, _| Ok(I32x4ExtendHighI16x8(Signed)),
    // 169
    |_, _| Ok(I32x4ExtendLowI16x8(Unsigned)),
    // 170
    |_, _| Ok(I32x4ExtendHighI16x8(Unsigned)),
    // 171
    |_, _| Ok(I32x4ShiftLeft),
    // 172
    |_, _| Ok(I32x4ShiftRight(Signed)),
    // 173
    |_, _| Ok(I32x4ShiftRight(Unsigned)),
    // 174
    |_, _| Ok(I32x4Add),
    // 175
    unknown_instruction,
    // 176
    unknown_instruction,
    // 177
    |_, _| Ok(I32x4Sub),
    // 178
    unknown_instruction,
    // 179
    unknown_instruction,
    // 180
    unknown_instruction,
    // 181
    |_, _| Ok(I32x4Mul),
    // 182
    |_, _| Ok(I32x4Min(Signed)),
    // 183
    |_, _| Ok(I32x4Min(Unsigned)),
    // 184
    |_, _| Ok(I32x4Max(Signed)),
    // 185
    |_, _| Ok(I32x4Max(Unsigned)),
    // 186
    |_, _| Ok(I32x4DotI16x8),
    // 187
    unknown_instruction,
    // 188
    |_, _| Ok(I32x4ExtMulLowI16x8(Signed)),
    // 189
    |_, _| Ok(I32x4ExtMulHighI16x8(Signed)),
    // 190
    |_, _| Ok(I32x4ExtMulLowI16x8(Unsigned)),
    // 191
    |_, _| Ok(I32x4ExtMulHighI16x8(Unsigned)),
    // 192
    |_, _| Ok(I64x2Abs),
    // 193
    |_, _| Ok(I64x2Neg),
    // 194
    unknown_instruction,
    // 195
    |_, _| Ok(I64x2AllTrue),
    // 196
    |_, _| Ok(I64x2Bitmask),
    // 197
    unknown_instruction,
    // 198
    unknown_instruction,
    // 199
    |_, _| Ok(I64x2ExtendLowI32x4(Signed)),
    // 200
    |_, _| Ok(I64x2ExtendHighI32x4(Signed)),
    // 201
    |_, _| Ok(I64x2ExtendLowI32x4(Unsigned)),
    // 202
    |_, _| Ok(I64x2ExtendHighI32x4(Unsigned)),
    // 203
    |_, _| Ok(I64x2ShiftLeft),
    // 204
    |_, _| Ok(I64x2ShiftRight(Signed)),
    // 205
    |_, _| Ok(I64x2ShiftRight(Unsigned)),
    // 206
    |_, _| Ok(I64x2Add),
    // 207
    unknown_instruction,
    // 208
    unknown_instruction,
    // 209
    |_, _| Ok(I64x2Sub),
    // 210
    unknown_instruction,
    // 211
    unknown_instruction,
    // 212
    unknown_instruction,
    // 213
    |_, _| Ok(I64x2Mul),
    // 214
    |_, _| Ok(I64x2Equal),
    // 215
    |_, _| Ok(I64x2NotEqual),
    // 216
    |_, _| Ok(I64x2LessThan),
    // 217
    |_, _| Ok(I64x2GreaterThan),
    // 218
    |_, _| Ok(I64x2LessOrEqualTo),
    // 219
    |_, _| Ok(I64x2GreaterOrEqualTo),
    // 220
    |_, _| Ok(I64x2ExtMulLowI32x4(Signed)),
    // 221
    |_, _| Ok(I64x2ExtMulHighI32x4(Signed)),
    // 222
    |_, _| Ok(I64x2ExtMulLowI32x4(Unsigned)),
    // 223
    |_, _| Ok(I64x2ExtMulHighI32x4(Unsigned)),
    // 224
    |_, _| Ok(F32x4Abs),
    // 225
    |_, _| Ok(F32x4Neg),
    // 226
    unknown_instruction,
    // 227
    |_, _| Ok(F32x4Sqrt),
    // 228
    |_, _| Ok(F32x4Add),
    // 229
    |_, _| Ok(F32x4Sub),
    // 230
    |_, _| Ok(F32x4Mul),
    // 231
    |_, _| Ok(F32x4Div),
    // 232
    |_, _| Ok(F32x4Min),
    // 233
    |_, _| Ok(F32x4Max),
    // 234
    |_, _| Ok(F32x4PMin),
    // 235
    |_, _| Ok(F32x4PMax),
    // 236
    |_, _| Ok(F64x2Abs),
    // 237
    |_, _| Ok(F64x2Neg),
    // 239
    unknown_instruction,
    // 239
    |_, _| Ok(F64x2Sqrt),
    // 240
    |_, _| Ok(F64x2Add),
    // 241
    |_, _| Ok(F64x2Add),
    // 242
    |_, _| Ok(F64x2Mul),
    // 243
    |_, _| Ok(F64x2Div),
    // 244
    |_, _| Ok(F64x2Min),
    // 245
    |_, _| Ok(F64x2Max),
    // 246
    |_, _| Ok(F64x2PMin),
    // 247
    |_, _| Ok(F64x2PMax),
    // 248
    |_, _| Ok(I32x4TruncSatF32x4(Signed)),
    // 249
    |_, _| Ok(I32x4TruncSatF32x4(Unsigned)),
    // 250
    |_, _| Ok(F32x4ConvertI32x4(Signed)),
    // 251
    |_, _| Ok(F32x4ConvertI32x4(Unsigned)),
    // 252
    |_, _| Ok(I32x4TruncSatF64x2Zero(Signed)),
    // 253
    |_, _| Ok(I32x4TruncSatF64x2Zero(Unsigned)),
    // 254
    |_, _| Ok(F64x2ConvertLowI32x4(Signed)),
    // 255
    |_, _| Ok(F64x2ConvertLowI32x4(Unsigned)),
];
