use edgedb_protocol::{
    descriptors::{Descriptor, TypePos},
    errors::DecodeError,
    queryable::{Decoder, DescriptorContext, DescriptorMismatch, Queryable},
    serialization::decode::DecodeTupleLike,
};

// The code below shows the code generated from the Queryable macro in a more readable form
// (with macro-generated qualified paths replaced with use statements).

#[derive(Debug)]
pub struct IsAStruct {
    pub name: String,
    pub number: i16,
    pub is_cool: bool,
}

impl Queryable for IsAStruct {
    fn decode(decoder: &Decoder, buf: &[u8]) -> Result<Self, DecodeError> {
        let nfields = 3usize
            + if decoder.has_implicit_id { 1 } else { 0 }
            + if decoder.has_implicit_tid { 1 } else { 0 }
            + if decoder.has_implicit_tname { 1 } else { 0 };
        let mut elements = DecodeTupleLike::new_object(buf, nfields)?;
        if decoder.has_implicit_tid {
            elements.skip_element()?;
        }
        if decoder.has_implicit_tname {
            elements.skip_element()?;
        }
        if decoder.has_implicit_id {
            elements.skip_element()?;
        }
        let name = Queryable::decode_optional(decoder, elements.read()?)?;
        let number = Queryable::decode_optional(decoder, elements.read()?)?;
        let is_cool = Queryable::decode_optional(decoder, elements.read()?)?;
        Ok(IsAStruct {
            name,
            number,
            is_cool,
        })
    }

    fn check_descriptor(
        ctx: &DescriptorContext,
        type_pos: TypePos,
    ) -> Result<(), DescriptorMismatch> {
        let desc = ctx.get(type_pos)?;
        let shape = match desc {
            Descriptor::ObjectShape(shape) => shape,
            _ => return Err(ctx.wrong_type(desc, "str")),
        };
        let mut idx = 0;
        if ctx.has_implicit_tid {
            if !shape.elements[idx].flag_implicit {
                return Err(ctx.expected("implicit __tid__"));
            }
            idx += 1;
        }
        if ctx.has_implicit_tname {
            if !shape.elements[idx].flag_implicit {
                return Err(ctx.expected("implicit __tname__"));
            }
            idx += 1;
        }
        if ctx.has_implicit_id {
            if !shape.elements[idx].flag_implicit {
                return Err(ctx.expected("implicit id"));
            }
            idx += 1;
        }
        let el = &shape.elements[idx];
        if el.name != "name" {
            return Err(ctx.wrong_field("name", &el.name));
        }
        idx += 1;
        <String as Queryable>::check_descriptor(ctx, el.type_pos)?;
        let el = &shape.elements[idx];
        if el.name != "number" {
            return Err(ctx.wrong_field("number", &el.name));
        }
        idx += 1;
        <i16 as Queryable>::check_descriptor(ctx, el.type_pos)?;
        let el = &shape.elements[idx];
        if el.name != "is_cool" {
            return Err(ctx.wrong_field("is_cool", &el.name));
        }
        idx += 1;
        <bool as Queryable>::check_descriptor(ctx, el.type_pos)?;
        if shape.elements.len() != idx {
            return Err(ctx.field_number(shape.elements.len(), idx));
        }
        Ok(())
    }
}
