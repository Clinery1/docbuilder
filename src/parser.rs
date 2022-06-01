use generic_parser::{
    GenericParser,
    Error,
    EOFError,
};
use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
};


pub trait Parser<'doc> {
    fn into_document(self)->Result<'doc,Document>;
    fn number(&mut self)->Result<'doc,f32>;
    fn name(&mut self)->Result<'doc,&'doc str>;
    fn size(&mut self)->Result<'doc,Size>;
    fn color(&mut self)->Result<'doc,Color>;
    fn direction(&mut self)->Result<'doc,Direction>;
    fn page_size(&mut self)->Result<'doc,PageSize>;
    fn vertical_text_align(&mut self)->Result<'doc,VTextAlign>;
    fn horizontal_text_align(&mut self)->Result<'doc,HTextAlign>;
    fn section(&mut self)->Result<'doc,Section>;
    fn section_style(&mut self)->Result<'doc,SectionStyle>;
    fn item(&mut self)->Result<'doc,Item>;
    fn metadata(&mut self)->Result<'doc,Metadata>;
    fn page(&mut self)->Result<'doc,Page>;
    fn page_style(&mut self)->Result<'doc,PageStyle>;
}
impl<'doc> Parser<'doc> for GenericParser<'doc,ErrorKind> {
    fn into_document(mut self)->Result<'doc,Document> {
        let metadata=self.skip(EXT_WHITESPACE).metadata()?;
        let mut pages=Vec::new();
        while !self.skip(EXT_WHITESPACE).is_eof() {
            pages.push(self.page()?);
        }
        return Ok(Document{metadata,pages});
    }
    fn number(&mut self)->Result<'doc,f32> {
        const NUMBERS:&[&str]=&[
            "1","2","3","4","5","6","7","8","9","0",
        ];
        let mut number=self.while_any(NUMBERS).to_string();
        if number.len()==0&&!self.test(".")? {
            return Err(self.create_error(ErrorKind::ExpectedNumber,false));
        }
        if self.then(".")? {
            number.push('.');
            number.push_str(self.while_any(NUMBERS));
        }
        match number.parse::<f32>() {
            Ok(num)=>Ok(num),
            Err(e)=>Err(self.create_error(ErrorKind::NumberParseError(e.to_string()),true)),
        }
    }
    fn name(&mut self)->Result<'doc,&'doc str> {
        let name=self.until_any(&[" ",":","{"]);
        if name.len()==0 {
            return Err(self.create_error(ErrorKind::ExpectedName,false));
        }
        return Ok(name);
    }
    fn size(&mut self)->Result<'doc,Size> {
        let num=self.number()?;
        if self.then("in")? {
            return Ok(Size::Inches(num));
        } else if self.then("pt")? {
            return Ok(Size::Points(num));
        } else if self.then("px")? {
            return Ok(Size::Pixels(num));
        } else if self.then("%")? {
            return Ok(Size::Percent(num));
        } else {
            return Err(self.create_error(ErrorKind::ExpectedSize,true));
        }
    }
    fn color(&mut self)->Result<'doc,Color> {
        const ALL_HEX_DIGITS:&[&str]=&[
            "1","2","3","4","5","6","7","8","9","0",
            "a","b","c","d","e","f","A","B","C","D","E","F",
        ];
        const HEX_LOOKUP:&str="0123456789abcdef";
        if !self.then("#")? {
            return Err(self.create_error(ErrorKind::ExpectedColor,false));
        }
        let mut hex_digits=self.while_any(ALL_HEX_DIGITS).to_lowercase();
        let len=hex_digits.len();
        let mut convert_hex_digit=|double|{
            if double {
                let num=(HEX_LOOKUP.find(hex_digits.remove(0)).unwrap() as u8)<<4;
                let num2=HEX_LOOKUP.find(hex_digits.remove(0)).unwrap() as u8;
                num+num2
            } else {
                let num=HEX_LOOKUP.find(hex_digits.remove(0)).unwrap() as u8;
                num+(num<<4)
            }
        };
        match len {
            3=>{
                let r=convert_hex_digit(false);
                let g=convert_hex_digit(false);
                let b=convert_hex_digit(false);
                return Ok(Color{r,g,b,a:None});
            },
            4=>{
                let r=convert_hex_digit(false);
                let g=convert_hex_digit(false);
                let b=convert_hex_digit(false);
                let a=convert_hex_digit(false);
                return Ok(Color{r,g,b,a:Some(a)});
            },
            6=>{
                let r=convert_hex_digit(true);
                let g=convert_hex_digit(true);
                let b=convert_hex_digit(true);
                return Ok(Color{r,g,b,a:None});
            },
            8=>{
                let r=convert_hex_digit(true);
                let g=convert_hex_digit(true);
                let b=convert_hex_digit(true);
                let a=convert_hex_digit(true);
                return Ok(Color{r,g,b,a:Some(a)});
            },
            _=>return Err(self.create_error(ErrorKind::InvalidColorLength,false)),
        }
    }
    fn direction(&mut self)->Result<'doc,Direction> {
        if self.then("Left")? {
            return Ok(Direction::Left);
        } else if self.then("Right")? {
            return Ok(Direction::Right);
        } else if self.then("Up")? {
            return Ok(Direction::Up);
        } else if self.then("Down")? {
            return Ok(Direction::Down);
        }
        return Err(self.create_error(ErrorKind::ExpectedDirection,false));
    }
    fn page_size(&mut self)->Result<'doc,PageSize> {
        if self.then(":")? {
            self.skip(WHITESPACE);
            if self.then("PortraitLetter")? {
                return Ok(PageSize::PortraitLetter);
            } else if self.then("LandscapeLetter")? {
                return Ok(PageSize::LandscapeLetter);
            } else if self.then("Webpage")? {
                return Ok(PageSize::Webpage);
            }
        } else if self.then("{")? {
            let mut width=None;
            let mut height=None;
            while !self.skip(EXT_WHITESPACE).then("}")? {
                let name=self.name()?;
                match name {
                    "width"=>{
                        if width.is_some() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Custom page size width"),true));
                        }
                        width=Some(self.size()?);
                    },
                    "height"=>{
                        if height.is_some() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Custom page size height"),true));
                        }
                        height=Some(self.size()?);
                    },
                    _=>return Err(self.create_error(ErrorKind::ExpectedPageSize,true)),
                }
            }
            let width=width.ok_or_else(||self.create_error(ErrorKind::ExpectedPageSizeWidth,true))?;
            let height=height.ok_or_else(||self.create_error(ErrorKind::ExpectedPageSizeHeight,true))?;
            return Ok(PageSize::Custom{width,height});
        }
        return Err(self.create_error(ErrorKind::ExpectedPageSize,true));
    }
    fn vertical_text_align(&mut self)->Result<'doc,VTextAlign> {
        if self.then("Top")? {
            return Ok(VTextAlign::Top);
        } else if self.then("Bottom")? {
            return Ok(VTextAlign::Bottom);
        } else if self.then("Center")? {
            return Ok(VTextAlign::Center);
        }
        return Err(self.create_error(ErrorKind::ExpectedTextAlign,true));
    }
    fn horizontal_text_align(&mut self)->Result<'doc,HTextAlign> {
        if self.then("Left")? {
            return Ok(HTextAlign::Left);
        } else if self.then("Rigth")? {
            return Ok(HTextAlign::Right);
        } else if self.then("Center")? {
            return Ok(HTextAlign::Center);
        }
        return Err(self.create_error(ErrorKind::ExpectedTextAlign,true));
    }
    fn section(&mut self)->Result<'doc,Section> {
        if !self.then("section")? {
            return Err(self.create_error(ErrorKind::ExpectedSection,false));
        }
        if !self.skip(WHITESPACE).then("{")? {
            return Err(self.create_error(ErrorKind::ExpectedSectionBlockStart,true));
        }
        let mut style=None;
        let mut content=None;
        while !self.skip(EXT_WHITESPACE).then("}")? {
            let mut sp=self.subparser();
            let name=sp.name()?;
            match name {
                "style"=>{
                    sp.finish_error();
                    if style.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section/style"),true));
                    }
                    style=Some(self.section_style()?);
                },
                "content"=>{
                    sp.finish();
                    if style.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section/style"),true));
                    }
                    if !self.skip(WHITESPACE).then("{")? {
                        return Err(self.create_error(ErrorKind::ExpectedSectionContentBlockStart,true));
                    }
                    self.skip(NEWLINE);
                    let initial_indent=self.while_any(WHITESPACE);
                    let mut lines=vec![self.until_any(NEWLINE).to_string()];
                    self.then_any(NEWLINE)?;
                    loop {
                        if !self.then(initial_indent)? {
                            break;
                        }
                        lines.push(self.until_any(NEWLINE).to_string());
                        self.skip(NEWLINE);
                    }
                    if !self.skip(WHITESPACE).then("}")? {
                        return Err(self.create_error(ErrorKind::ExpectedSectionContentBlockEnd,true));
                    }
                    content=Some(lines);
                },
                _=>{
                    sp.finish_error();
                    return Err(self.create_error(ErrorKind::ExpectedSection,true));
                },
            }
        }
        let content=content.ok_or_else(||self.create_error(ErrorKind::ExpectedSectionContent,true))?;
        return Ok(Section{content,style});
    }
    fn section_style(&mut self)->Result<'doc,SectionStyle> {
        if !self.then("style")? {
            return Err(self.create_error(ErrorKind::ExpectedSectionStyle,false));
        }
        if !self.skip(WHITESPACE).then("{")? {
            return Err(self.create_error(ErrorKind::ExpectedSectionStyleBlockStart,true));
        }
        let mut width=None;
        let mut height=None;
        let mut align=None;
        let mut font=None;
        let mut font_size=None;
        let mut text_color=None;
        let mut background_color=None;
        let mut margin:Option<SizedSides>=None;
        let mut vertical_text_align=None;
        let mut horizontal_text_align=None;
        while !self.skip(EXT_WHITESPACE).then("}")? {
            let name=self.name()?;
            if !self.then(":")? {
                return Err(self.create_error(ErrorKind::ExpectedColon,true));
            }
            self.skip(WHITESPACE);
            match name {
                "width"=>{
                    if width.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/width"),true));
                    }
                    width=Some(self.size()?);
                },
                "height"=>{
                    if height.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/height"),true));
                    }
                    height=Some(self.size()?);
                },
                "vertical_text_align"=>{
                    if vertical_text_align.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/vertical text align"),true));
                    }
                    vertical_text_align=Some(self.vertical_text_align()?);
                },
                "horizontal_text_align"=>{
                    if horizontal_text_align.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/horizontal text align"),true));
                    }
                    horizontal_text_align=Some(self.horizontal_text_align()?);
                },
                "align"=>{
                    if align.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/align"),true));
                    }
                    align=Some(self.direction()?);
                },
                "font"=>{
                    if font.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/font"),true));
                    }
                    font=Some(self.until_any(NEWLINE).to_string());
                },
                "font_size"=>{
                    if font_size.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/font size"),true));
                    }
                    font_size=Some(self.size()?);
                },
                "text_color"=>{
                    if text_color.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/text color"),true));
                    }
                    text_color=Some(self.color()?);
                },
                "background_color"=>{
                    if background_color.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/background color"),true));
                    }
                    background_color=Some(self.color()?);
                },
                "margin"=>{
                    if let Some(margin)=&mut margin {
                        if margin.is_individual() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin-*"),true));
                        }
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                    } else {
                        margin=Some(SizedSides::All(self.size()?));
                    }
                },
                "margin_left"=>{
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_left_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_left"),true));
                        }
                        margin.set_left(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:Some(self.size()?),
                            right:None,
                            top:None,
                            bottom:None,
                        });
                    }
                },
                "margin_right"=>{
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_right_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_right"),true));
                        }
                        margin.set_right(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:None,
                            right:Some(self.size()?),
                            top:None,
                            bottom:None,
                        });
                    }
                },
                "margin_top"=>{
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_top_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_top"),true));
                        }
                        margin.set_top(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:None,
                            right:None,
                            top:Some(self.size()?),
                            bottom:None,
                        });
                    }
                },
                "margin_bottom"=>{
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_bottom_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_bottom"),true));
                        }
                        margin.set_bottom(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:None,
                            right:None,
                            top:None,
                            bottom:Some(self.size()?),
                        });
                    }
                },
                _=>return Err(self.create_error(ErrorKind::ExpectedSectionStyle,true)),
            }
        }
        return Ok(SectionStyle{width,height,align,font,font_size,text_color,background_color,margin,vertical_text_align,horizontal_text_align});
    }
    fn item(&mut self)->Result<'doc,Item> {
        if self.then("vertical")? {
            if !self.skip(WHITESPACE).then("{")? {
                return Err(self.create_error(ErrorKind::ExpectedItemBlockStart,true));
            }
            let mut items=Vec::new();
            let mut style=None;
            while !self.skip(EXT_WHITESPACE).then("}")? {
                let mut sp=self.subparser();
                let name=sp.name()?;
                sp.finish_error();
                match name {
                    "style"=>{
                        if style.is_some() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Item/style"),true));
                        }
                        style=Some(self.section_style()?);
                    },
                    _=>items.push(self.item()?),
                }
            }
            return Ok(Item::Vertical{items,style});
        } else if self.then("horizontal")? {
            if !self.skip(WHITESPACE).then("{")? {
                return Err(self.create_error(ErrorKind::ExpectedItemBlockStart,true));
            }
            let mut items=Vec::new();
            let mut style=None;
            while !self.skip(EXT_WHITESPACE).then("}")? {
                let mut sp=self.subparser();
                let name=sp.name()?;
                sp.finish_error();
                match name {
                    "style"=>{
                        if style.is_some() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Item/style"),true));
                        }
                        style=Some(self.section_style()?);
                    },
                    _=>items.push(self.item()?),
                }
            }
            return Ok(Item::Horizontal{items,style});
        } else if self.test("section")? {
            return Ok(Item::Section(self.section()?));
        }
        return Err(self.create_error(ErrorKind::ExpectedItem,false));
    }
    fn metadata(&mut self)->Result<'doc,Metadata> {
        if !self.then("metadata")? {
            return Err(self.create_error(ErrorKind::ExpectedMetadata,false));
        }
        if !self.skip(WHITESPACE).then("{")? {
            return Err(self.create_error(ErrorKind::ExpectedMetadataBlockStart,true));
        }
        let mut title=None;
        let mut page_style=None;
        while !self.skip(EXT_WHITESPACE).then("}")? {
            let mut sp=self.subparser();
            let name=sp.name()?;
            match name {
                "title"=>{
                    sp.finish();
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if title.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Metadata/title"),true));
                    }
                    title=Some(self.until_any(NEWLINE).to_string());
                },
                "style"=>{
                    sp.finish_error();
                    if page_style.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Metadata/style"),true));
                    }
                    page_style=Some(self.page_style()?);
                },
                _=>{
                    sp.finish_error();
                    return Err(self.create_error(ErrorKind::ExpectedMetadata,true));
                },
            }
        }
        let title=title.ok_or_else(||self.create_error(ErrorKind::ExpectedMetadataTitle,true))?;
        return Ok(Metadata{title,page_style});
    }
    fn page(&mut self)->Result<'doc,Page> {
        if !self.then("page")? {
            return Err(self.create_error(ErrorKind::ExpectedPage,false));
        }
        if !self.skip(WHITESPACE).then("{")? {
            return Err(self.create_error(ErrorKind::ExpectedPageBlockStart,true));
        }
        let mut items=Vec::new();
        let mut style=None;
        while !self.skip(EXT_WHITESPACE).then("}")? {
            let mut sp=self.subparser();
            let name=sp.name()?;
            sp.finish_error();
            match name {
                "style"=>{
                    if style.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Page/style"),true));
                    }
                    style=Some(self.page_style()?);
                },
                _=>{
                    match self.item() {
                        Ok(item)=>{
                            items.push(item);
                            continue;
                        },
                        Err(e)=>{
                            if e.important {
                                return Err(e);
                            }
                        },
                    }
                    return Err(self.create_error(ErrorKind::ExpectedPage,true));
                },
            }
        }
        return Ok(Page{items,style});
    }
    fn page_style(&mut self)->Result<'doc,PageStyle> {
        if !self.then("style")? {
            return Err(self.create_error(ErrorKind::ExpectedPageStyle,false));
        }
        if !self.skip(WHITESPACE).then("{")? {
            return Err(self.create_error(ErrorKind::ExpectedPageStyleBlockStart,true));
        }
        let mut page_size=None;
        let mut text_color=None;
        let mut background_color=None;
        let mut margin:Option<SizedSides>=None;
        let mut vertical_text_align=None;
        let mut horizontal_text_align=None;
        while !self.skip(EXT_WHITESPACE).then("}")? {
            let name=self.name()?;
            match name {
                "page_size"=>{
                    if page_size.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/page size"),true));
                    }
                    self.skip(WHITESPACE);
                    page_size=Some(self.page_size()?);
                },
                "text_color"=>{
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if text_color.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/text color"),true));
                    }
                    text_color=Some(self.color()?);
                },
                "background_color"=>{
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if background_color.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/background color"),true));
                    }
                    background_color=Some(self.color()?);
                },
                "vertical_text_align"=>{
                    if vertical_text_align.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/vertical text align"),true));
                    }
                    vertical_text_align=Some(self.vertical_text_align()?);
                },
                "horizontal_text_align"=>{
                    if horizontal_text_align.is_some() {
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Section style/horizontal text align"),true));
                    }
                    horizontal_text_align=Some(self.horizontal_text_align()?);
                },
                "margin"=>{
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if let Some(margin)=&mut margin {
                        if margin.is_individual() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin-*"),true));
                        }
                        return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                    } else {
                        margin=Some(SizedSides::All(self.size()?));
                    }
                },
                "margin_left"=>{
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_left_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_left"),true));
                        }
                        margin.set_left(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:Some(self.size()?),
                            right:None,
                            top:None,
                            bottom:None,
                        });
                    }
                },
                "margin_right"=>{
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_right_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_right"),true));
                        }
                        margin.set_right(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:None,
                            right:Some(self.size()?),
                            top:None,
                            bottom:None,
                        });
                    }
                },
                "margin_top"=>{
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_top_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_top"),true));
                        }
                        margin.set_top(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:None,
                            right:None,
                            top:Some(self.size()?),
                            bottom:None,
                        });
                    }
                },
                "margin_bottom"=>{
                    if !self.then(":")? {
                        return Err(self.create_error(ErrorKind::ExpectedColon,true));
                    }
                    self.skip(WHITESPACE);
                    if let Some(margin)=&mut margin {
                        if margin.is_all() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin"),true));
                        } else if margin.is_bottom_defined() {
                            return Err(self.create_error(ErrorKind::AlreadyDefined("Page style/margin_bottom"),true));
                        }
                        margin.set_bottom(self.size()?);
                    } else {
                        margin=Some(SizedSides::Individual {
                            left:None,
                            right:None,
                            top:None,
                            bottom:Some(self.size()?),
                        });
                    }
                },
                _=>return Err(self.create_error(ErrorKind::ExpectedPageStyle,true)),
            }
        }
        return Ok(PageStyle{page_size,text_color,background_color,margin,vertical_text_align,horizontal_text_align});
    }
}


type Result<'doc,T>=std::result::Result<T,Error<'doc,ErrorKind>>;


#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedEof,
    ExpectedNumber,
    ExpectedName,
    ExpectedSize,
    ExpectedColor,
    ExpectedPageSize,
    ExpectedDirection,
    ExpectedPageSizeWidth,
    ExpectedPageSizeHeight,
    ExpectedSectionStyle,
    ExpectedColon,
    ExpectedPageStyle,
    ExpectedPageStyleBlockStart,
    ExpectedMetadata,
    ExpectedMetadataBlockStart,
    ExpectedMetadataTitle,
    ExpectedSectionContent,
    ExpectedSectionContentBlockStart,
    ExpectedSectionContentBlockEnd,
    ExpectedPage,
    ExpectedPageBlockStart,
    ExpectedSection,
    ExpectedSectionBlockStart,
    ExpectedSectionStyleBlockStart,
    ExpectedItem,
    ExpectedItemBlockStart,
    ExpectedTextAlign,
    InvalidColorLength,
    AlreadyDefined(&'static str),
    NumberParseError(String),
}
impl Display for ErrorKind {
    fn fmt(&self,f:&mut Formatter)->FmtResult {
        use ErrorKind::*;
        match self {
            UnexpectedEof=>write!(f,"Unexpected EOF"),
            ExpectedNumber=>write!(f,"Expected number"),
            ExpectedName=>write!(f,"Expected name"),
            ExpectedSize=>write!(f,"Expected size"),
            ExpectedColor=>write!(f,"Expected color in HTML hex format: `#NNN` where there are 3, 4, 6, or 8 `N`s"),
            ExpectedPageSize=>write!(f,"Expected page size"),
            ExpectedDirection=>write!(f,"Expected direction"),
            ExpectedPageSizeWidth=>write!(f,"Expected page style width"),
            ExpectedPageSizeHeight=>write!(f,"Expected page style height"),
            ExpectedSectionStyle=>write!(f,"Expected section style"),
            ExpectedColon=>write!(f,"Expected `:`"),
            ExpectedPageStyle=>write!(f,"Expected page style"),
            ExpectedPageStyleBlockStart=>write!(f,"Expected page style block start (`{{`)"),
            ExpectedMetadata=>write!(f,"Expected metadata"),
            ExpectedMetadataBlockStart=>write!(f,"Expected metadata block start (`{{`)"),
            ExpectedMetadataTitle=>write!(f,"Expected title"),
            ExpectedSectionContent=>write!(f,"Expected section content"),
            ExpectedSectionContentBlockStart=>write!(f,"Expected content block start (`{{`)"),
            ExpectedSectionContentBlockEnd=>write!(f,"Expected content block end (`}}`)"),
            ExpectedPage=>write!(f,"Expected page"),
            ExpectedPageBlockStart=>write!(f,"Expected page block start (`{{`)"),
            ExpectedSection=>write!(f,"Expected section"),
            ExpectedSectionBlockStart=>write!(f,"Expected section block start (`{{`)"),
            ExpectedSectionStyleBlockStart=>write!(f,"Expected section style block start (`{{`)"),
            ExpectedItem=>write!(f,"Expected horizontal, vertical, or section"),
            ExpectedItemBlockStart=>write!(f,"Expected horizontal, vertical, or section block start (`{{`)"),
            ExpectedTextAlign=>write!(f,"Expected text align"),
            InvalidColorLength=>write!(f,"Invalid hex code length. Expected 3, 4, 6, or 8 digits."),
            AlreadyDefined(item)=>write!(f,"{} is already defined",item),
            NumberParseError(s)=>write!(f,"Error parsing number: {}",s),
        }
    }
}
impl EOFError for ErrorKind {
    fn create_eof()->Self {ErrorKind::UnexpectedEof}
}
#[derive(Debug)]
pub enum Item {
    Vertical {
        items:Vec<Self>,
        style:Option<SectionStyle>,
    },
    Horizontal {
        items:Vec<Self>,
        style:Option<SectionStyle>,
    },
    Section(Section),
}
#[derive(Debug)]
pub enum Size {
    Inches(f32),
    Points(f32),
    Pixels(f32),
    Percent(f32),
}
#[derive(Debug)]
pub enum PageSize {
    PortraitLetter,
    LandscapeLetter,
    /// Has no defined size. Probably counts as responsive design.
    Webpage,
    Custom {
        width:Size,
        height:Size,
    },
}
#[derive(Debug)]
pub enum SizedSides {
    All(Size),
    Individual {
        left:Option<Size>,
        right:Option<Size>,
        top:Option<Size>,
        bottom:Option<Size>,
    },
}
impl SizedSides {
    pub fn is_all(&self)->bool {
        match self {
            Self::All(_)=>true,
            _=>false,
        }
    }
    pub fn is_individual(&self)->bool {
        match self {
            Self::Individual{..}=>true,
            _=>false,
        }
    }
    pub fn is_left_defined(&self)->bool {
        match self {
            Self::Individual{left,..}=>left.is_some(),
            _=>false,
        }
    }
    pub fn is_right_defined(&self)->bool {
        match self {
            Self::Individual{right,..}=>right.is_some(),
            _=>false,
        }
    }
    pub fn is_top_defined(&self)->bool {
        match self {
            Self::Individual{top,..}=>top.is_some(),
            _=>false,
        }
    }
    pub fn is_bottom_defined(&self)->bool {
        match self {
            Self::Individual{bottom,..}=>bottom.is_some(),
            _=>false,
        }
    }
    pub fn set_left(&mut self,size:Size) {
        match self {
            Self::Individual{left,..}=>*left=Some(size),
            _=>{},
        }
    }
    pub fn set_right(&mut self,size:Size) {
        match self {
            Self::Individual{right,..}=>*right=Some(size),
            _=>{},
        }
    }
    pub fn set_top(&mut self,size:Size) {
        match self {
            Self::Individual{top,..}=>*top=Some(size),
            _=>{},
        }
    }
    pub fn set_bottom(&mut self,size:Size) {
        match self {
            Self::Individual{bottom,..}=>*bottom=Some(size),
            _=>{},
        }
    }
}
#[derive(Debug)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
#[derive(Debug,PartialEq,Copy,Clone)]
pub enum HTextAlign {
    Left,
    Right,
    Center,
}
#[derive(Debug,PartialEq,Copy,Clone)]
pub enum VTextAlign {
    Top,
    Bottom,
    Center,
}


#[derive(Debug)]
pub struct Document {
    pub metadata:Metadata,
    pub pages:Vec<Page>,
}
#[derive(Debug)]
pub struct Metadata {
    pub title:String,
    pub page_style:Option<PageStyle>,
}
#[derive(Debug)]
pub struct Page {
    pub items:Vec<Item>,
    pub style:Option<PageStyle>,
}
#[derive(Debug)]
pub struct PageStyle {
    pub page_size:Option<PageSize>,
    pub text_color:Option<Color>,
    pub background_color:Option<Color>,
    pub margin:Option<SizedSides>,
    pub horizontal_text_align:Option<HTextAlign>,
    pub vertical_text_align:Option<VTextAlign>,
}
impl Default for PageStyle {
    fn default()->Self {
        Self {
            page_size:Some(PageSize::PortraitLetter),
            text_color:None,
            background_color:None,
            margin:None,
            horizontal_text_align:None,
            vertical_text_align:None,
        }
    }
}
#[derive(Debug)]
pub struct Section {
    pub style:Option<SectionStyle>,
    pub content:Vec<String>,
}
#[derive(Debug,Default)]
pub struct SectionStyle {
    pub width:Option<Size>,
    pub height:Option<Size>,
    pub align:Option<Direction>,
    pub font:Option<String>,
    pub font_size:Option<Size>,
    pub text_color:Option<Color>,
    pub background_color:Option<Color>,
    pub margin:Option<SizedSides>,
    pub horizontal_text_align:Option<HTextAlign>,
    pub vertical_text_align:Option<VTextAlign>,
}
#[derive(Debug)]
pub struct Color {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub a:Option<u8>,
}


const WHITESPACE:&[&str]=&[
    " ",
];
const EXT_WHITESPACE:&[&str]=&[
    " ",
    "\r","\n",
];
const NEWLINE:&[&str]=&[
    "\n","\r\n",
];
