use super::Instruction;

impl Instruction {
    pub fn get_child_instructions(&self) -> ChildInstructions {
        match self {
            Self::Block { inner, .. } => ChildInstructions::One(inner),
            Self::Loop { inner, .. } => ChildInstructions::One(inner),
            Self::If { inner, .. } => ChildInstructions::One(inner),
            Self::IfElse {
                if_inner,
                else_inner,
                ..
            } => ChildInstructions::Two(if_inner, else_inner),
            _ => ChildInstructions::None,
        }
    }
}

pub enum ChildInstructions<'a> {
    None,
    One(&'a [Instruction]),
    Two(&'a [Instruction], &'a [Instruction]),
}

impl<'a> ChildInstructions<'a> {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}
