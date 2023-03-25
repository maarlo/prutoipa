use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crate::{
    descriptor::enum_descriptor::{EnumDescriptor, EnumValue},
    error::PrutoipaBuildError,
    generator::Indent,
};

pub fn generate_enum(
    writer: &mut BufWriter<File>,
    package_name: String,
    name: String,
    enum_descriptor: EnumDescriptor,
    generate_enum_values: bool,
) -> Result<(), PrutoipaBuildError> {
    write_head(writer, package_name, name)?;
    let res = write_values(writer, enum_descriptor.get_values(), generate_enum_values);
    write_tail(writer)?;

    res
}

fn write_head(
    writer: &mut BufWriter<File>,
    package_name: String,
    name: String,
) -> Result<(), PrutoipaBuildError> {
    let i_00 = Indent(0);
    let i_04 = Indent(1);
    let i_08 = Indent(2);
    let i_12 = Indent(3);
    let i_16 = Indent(4);
    let i_20 = Indent(5);

    let lines_to_write = vec![
        format!("{i_00}impl<'__s> utoipa::ToSchema<'__s> for {name} {{"),
        format!("{i_04}fn schema() -> (&'__s str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {{"),
        format!("{i_08}("),
        format!("{i_12}\"{package_name}::{name}\","),
        format!("{i_12}utoipa::openapi::ObjectBuilder::new()"),
        format!("{i_16}.schema_type(utoipa::openapi::SchemaType::Integer)"),
        format!("{i_16}.format(Some(utoipa::openapi::SchemaFormat::KnownFormat("),
        format!("{i_20}utoipa::openapi::KnownFormat::Int32"),
        format!("{i_16})))"),
    ];

    lines_to_write
        .into_iter()
        .map(|line| writeln!(writer, "{line}"))
        .collect::<Result<Vec<()>, std::io::Error>>()?;

    Ok(())
}

fn write_tail(writer: &mut BufWriter<File>) -> Result<(), PrutoipaBuildError> {
    let i_00 = Indent(0);
    let i_04 = Indent(1);
    let i_08 = Indent(2);
    let i_16 = Indent(4);

    let lines_to_write = vec![
        format!("{i_16}.into()"),
        format!("{i_08})"),
        format!("{i_04}}}"),
        format!("{i_00}}}"),
    ];

    lines_to_write
        .into_iter()
        .map(|line| writeln!(writer, "{line}"))
        .collect::<Result<Vec<()>, std::io::Error>>()?;

    Ok(())
}

fn write_values(
    writer: &mut BufWriter<File>,
    enum_values: Vec<EnumValue>,
    generate_enum_values: bool,
) -> Result<(), PrutoipaBuildError> {
    let i_16 = Indent(4);
    let i_20 = Indent(5);

    //
    let mut description = format!("{i_16}.description(Some(\"Values:");
    enum_values.clone().into_iter().for_each(|enum_value| {
        description.push_str(format!("\\n\\n{} = {}", enum_value.number, enum_value.name).as_str());
    });
    description.push_str("\"))");

    writeln!(writer, "{description}")?;

    //
    if generate_enum_values {
        let mut enum_values_lines = Vec::<String>::new();
        enum_values_lines.push(format!("{i_16}.enum_values(Some(vec!["));
        enum_values.into_iter().for_each(|enum_value| {
            enum_values_lines.push(format!("{i_20}\"{}\",", enum_value.number));
        });
        enum_values_lines.push(format!("{i_16}]))"));

        enum_values_lines
            .into_iter()
            .map(|line| writeln!(writer, "{line}"))
            .collect::<Result<Vec<()>, std::io::Error>>()?;
    }

    Ok(())
}
