use pulldown_cmark::{
    Parser,
    Event,
    Tag,
    Options,
    CodeBlockKind,
    HeadingLevel,
    LinkType,
};
use crate::parser::{
    Document,
    Metadata,
    Page,
    PageStyle,
    Section,
    SectionStyle,
    Item,
    Size,
    Color,
    PageSize,
    SizedSides,
    Direction,
    VTextAlign,
    HTextAlign,
};


pub trait IntoHtml {
    fn into_html(self,parent_direction:ParentDirection)->String;
}
impl IntoHtml for Size {
    fn into_html(self,_:ParentDirection)->String {
        use Size::*;
        match self {
            Inches(c)=>format!("{}in",c),
            Points(c)=>format!("{}pt",c),
            Pixels(c)=>format!("{}px",c),
            Percent(c)=>format!("{}%",c),
        }
    }
}
impl IntoHtml for Color {
    fn into_html(self,_:ParentDirection)->String {
        format!("#{:02X}{:02X}{:02X}{:02X}",self.r,self.g,self.b,self.a.unwrap_or(0xff))
    }
}
impl IntoHtml for Direction {
    fn into_html(self,_:ParentDirection)->String {
        use Direction::*;
        match self {
            Left=>"left",
            Right=>"right",
            Up=>"top",
            Down=>"bottom",
        }.to_string()
    }
}
impl IntoHtml for PageSize {
    fn into_html(self,parent_direction:ParentDirection)->String {
        use PageSize::*;
        match self {
            PortraitLetter=>format!("width:8.5in;height:11in;"),
            LandscapeLetter=>format!("width:11in;height:8.5in;"),
            Webpage=>format!("width:100%;height:100%;"),
            Custom{width,height}=>format!("width:{};height:{};",width.into_html(parent_direction),height.into_html(parent_direction)),
        }
    }
}
impl IntoHtml for SizedSides {
    fn into_html(self,parent_direction:ParentDirection)->String {
        use SizedSides::*;
        match self {
            All(size)=>{
                return format!("NAME:{};",size.into_html(parent_direction));
            },
            Individual{left,right,top,bottom}=>{
                let mut out=String::new();
                if let Some(size)=left {
                    let fmt=format!("NAME-left:{};",size.into_html(parent_direction));
                    out.push_str(&fmt);
                }
                if let Some(size)=right {
                    let fmt=format!("NAME-right:{};",size.into_html(parent_direction));
                    out.push_str(&fmt);
                }
                if let Some(size)=top {
                    let fmt=format!("NAME-top:{};",size.into_html(parent_direction));
                    out.push_str(&fmt);
                }
                if let Some(size)=bottom {
                    let fmt=format!("NAME-bottom:{};",size.into_html(parent_direction));
                    out.push_str(&fmt);
                }
                return out;
            },
        }
    }
}
impl IntoHtml for VTextAlign {
    fn into_html(self,_:ParentDirection)->String {
        use VTextAlign::*;
        match self {
            Top=>"top",
            Bottom=>"bottom",
            Center=>"center",
        }.to_string()
    }
}
impl IntoHtml for HTextAlign {
    fn into_html(self,_:ParentDirection)->String {
        use HTextAlign::*;
        match self {
            Left=>"left",
            Right=>"right",
            Center=>"center",
        }.to_string()
    }
}
impl IntoHtml for SectionStyle {
    fn into_html(self,parent_direction:ParentDirection)->String {
        let mut out=String::new();
        out.push_str("padding:0;");
        out.push_str("margin:0;");
        out.push_str("overflow-wrap:anywhere;");
        if self.width.is_none()&&self.height.is_none() {
            out.push_str("flex-grow:1;");
            out.push_str("flex-shrink:1;");
        } else {
            out.push_str("flex-grow:0;");
            out.push_str("flex-shrink:0;");
        }
        if let Some(width)=self.width {
            let fmt=format!("width:{};",width.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(height)=self.height {
            let fmt=format!("height:{};",height.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(align)=self.align {
            let fmt=format!("float:{};",align.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(font)=self.font {
            let fmt=format!("font-family:{};",font);
            out.push_str(&fmt);
        }
        if let Some(font_size)=self.font_size {
            let fmt=format!("font-size:{};",font_size.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(text_color)=self.text_color {
            let fmt=format!("color:{};",text_color.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(background_color)=self.background_color {
            let fmt=format!("background-color:{};",background_color.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(margin)=self.margin {
            let fmt=margin.into_html(parent_direction).replace("NAME","margin");
            out.push_str(&fmt);
        }
        let mut need_flexbox=false;
        if let Some(align)=self.vertical_text_align {
            let fmt=if align==VTextAlign::Center {
                need_flexbox=true;
                if parent_direction==ParentDirection::Horizontal {
                    format!("align-items:{};",align.into_html(parent_direction))
                } else {
                    format!("justify-content:{};",align.into_html(parent_direction))
                }
            } else {
                format!("vertical-align:{};",align.into_html(parent_direction))
            };
            out.push_str(&fmt);
        }
        if let Some(align)=self.horizontal_text_align {
            let fmt=if align==HTextAlign::Center {
                need_flexbox=true;
                if parent_direction==ParentDirection::Horizontal {
                    format!("justify-content:{};",align.into_html(parent_direction))
                } else {
                    format!("align-items:{};",align.into_html(parent_direction))
                }
            } else {
                format!("text-align:{};",align.into_html(parent_direction))
            };
            out.push_str(&fmt);
        }
        if need_flexbox {
            out.push_str("display:flex;");
        }
        return out;
    }
}
impl IntoHtml for Section {
    fn into_html(self,parent_direction:ParentDirection)->String {
        let mut size_style=SectionStyle::default();
        let mut self_style=self.style.unwrap_or_default();
        size_style.width=self_style.width.take();
        size_style.height=self_style.height.take();
        size_style.align=self_style.align.take();
        size_style.vertical_text_align=self_style.vertical_text_align.clone();
        size_style.horizontal_text_align=self_style.horizontal_text_align.clone();
        size_style.background_color=self_style.background_color.take();
        let mut out=format!("<div style=\"{}\"><div style=\"{}\">",size_style.into_html(parent_direction),self_style.into_html(parent_direction));
        let mut source=String::new();
        for s in self.content {
            source.push_str(&s);
            source.push('\n');
        }
        let mut code:Option<(Option<String>,String)>=None;
        //println!("-------------------------------");
        for event in Parser::new_ext(&source,Options::ENABLE_STRIKETHROUGH) {
            //println!("Markdown item: {:?}",event);
            use Event::*;
            match event {
                Start(tag)=>{
                    use Tag::*;
                    match tag {
                        Paragraph=>out.push_str("<p>"),
                        BlockQuote=>out.push_str("<blockquote>"),
                        List(start)=>{
                            if let Some(start)=start {
                                let fmt=format!("<ol start=\"{}\">",start);
                                out.push_str(&fmt);
                            } else {
                                out.push_str("<ul>");
                            }
                        },
                        Item=>out.push_str("<li>"),
                        Heading(level,_,_)=>{
                            use HeadingLevel::*;
                            match level {
                                H1=>out.push_str("<h1>"),
                                H2=>out.push_str("<h2>"),
                                H3=>out.push_str("<h3>"),
                                H4=>out.push_str("<h4>"),
                                H5=>out.push_str("<h5>"),
                                H6=>out.push_str("<h6>"),
                            }
                        },
                        CodeBlock(ty)=>{
                            use CodeBlockKind::*;
                            let ty=match ty {
                                Fenced(lang)=>Some(lang.to_string()),
                                _=>None,
                            };
                            code=Some((ty,String::new()));
                        },
                        Emphasis=>out.push_str("<em>"),
                        Strong=>out.push_str("<strong>"),
                        Strikethrough=>out.push_str("<strike>"),
                        Link(ty,dest,title)=>{
                            if ty!=LinkType::Inline {
                                println!("Warning: only inline link type is supported");
                                continue;
                            }
                            let fmt=format!("<a href=\"{}\" title=\"{}\">",&*dest,&*title);
                            out.push_str(&fmt);
                        },
                        Image(ty,dest,title)=>{
                            if ty!=LinkType::Inline {
                                println!("Warning: only inline link type is supported");
                                continue;
                            }
                            let fmt=format!("<img src=\"{}\" title=\"{}\">",&*dest,&*title);
                            out.push_str(&fmt);
                        },
                        FootnoteDefinition(name)=>{
                            let fmt=format!("<span id=\"{}\">",name);
                            out.push_str(&fmt);
                        },
                        _=>{},
                    }
                },
                End(tag)=>{
                    use Tag::*;
                    match tag {
                        Paragraph=>out.push_str("</p>"),
                        BlockQuote=>out.push_str("</blockquote>"),
                        List(start)=>{
                            if start.is_some() {
                                out.push_str("</ol>");
                            } else {
                                out.push_str("</ul>");
                            }
                        },
                        Item=>out.push_str("</li>"),
                        Heading(level,_,_)=>{
                            use HeadingLevel::*;
                            match level {
                                H1=>out.push_str("</h1>"),
                                H2=>out.push_str("</h2>"),
                                H3=>out.push_str("</h3>"),
                                H4=>out.push_str("</h4>"),
                                H5=>out.push_str("</h5>"),
                                H6=>out.push_str("</h6>"),
                            }
                        },
                        CodeBlock(_)=>{
                            // TODO: code block highlighting
                            let code=code.take().unwrap();
                            out.push_str("<pre style=\"white-space:break-spaces\">");
                            out.push_str(&code.1);
                            out.push_str("</pre>");
                        },
                        Emphasis=>out.push_str("</em>"),
                        Strong=>out.push_str("</strong>"),
                        Strikethrough=>out.push_str("</strike>"),
                        Link(..)=>out.push_str("</a>"),
                        FootnoteDefinition(_)=>out.push_str("</span>"),
                        _=>{},
                    }
                },
                Text(text)=>{
                    if let Some((_,code_text))=&mut code {
                        code_text.push_str(&*text);
                    } else {
                        out.push_str(&*text);
                    }
                },
                Code(code)=>{
                    out.push_str("<span style=\"font-family:monospace\">");
                    out.push_str(&*code);
                    out.push_str("</span>");
                },
                Html(html)=>out.push_str(&*html),
                FootnoteReference(r)=>{
                    let fmt=format!("<a href=\"#{}\"><sup>{}</sup></a>",r,r);
                    out.push_str(&fmt)
                },
                SoftBreak=>out.push('\n'),
                HardBreak=>out.push_str("<br>"),
                Rule=>out.push_str("<hr>"),
                _=>{},
            }
        }
        //println!("-------------------------------");
        out.push_str("</div></div>");
        return out;
    }
}
impl IntoHtml for Item {
    fn into_html(self,parent_direction:ParentDirection)->String {
        use Item::*;
        match self {
            Horizontal{items,style}=>{
                let mut out=format!("<div style=\"{}align-items:stretch;display:flex;flex-direction:row;\">",style.unwrap_or_default().into_html(ParentDirection::Horizontal));
                for item in items {
                    let fmt=item.into_html(ParentDirection::Horizontal);
                    out.push_str(&fmt);
                }
                out.push_str("</div>");
                return out;
            },
            Vertical{items,style}=>{
                let mut out=format!("<div style=\"{}align-items:stretch;display:flex;flex-direction:column;\">",style.unwrap_or_default().into_html(ParentDirection::Vertical));
                for item in items {
                    let fmt=item.into_html(ParentDirection::Vertical);
                    out.push_str(&fmt);
                }
                out.push_str("</div>");
                return out;
            },
            Section(s)=>return s.into_html(parent_direction),
        }
    }
}
impl IntoHtml for PageStyle {
    fn into_html(self,parent_direction:ParentDirection)->String {
        let mut out=String::new();
        if let Some(page_size)=self.page_size {
            let fmt=page_size.into_html(parent_direction);
            out.push_str(&fmt);
        }
        if let Some(text_color)=self.text_color {
            let fmt=format!("color:{};",text_color.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(background_color)=self.background_color {
            let fmt=format!("background-color:{};",background_color.into_html(parent_direction));
            out.push_str(&fmt);
        }
        if let Some(margin)=self.margin {
            let fmt=margin.into_html(parent_direction).replace("NAME","padding");
            out.push_str(&fmt);
        }
        out.push_str("margin:0;");
        return out;
    }
}
impl IntoHtml for Page {
    fn into_html(self,parent_direction:ParentDirection)->String {
        let mut out=format!("<div style=\"{}\" class=\"page\">",self.style.unwrap_or_default().into_html(parent_direction));
        for item in self.items {
            let fmt=item.into_html(parent_direction);
            out.push_str(&fmt);
        }
        out.push_str("</div>");
        return out;
    }
}
impl IntoHtml for Metadata {
    fn into_html(self,parent_direction:ParentDirection)->String {
        return format!("<title>{}</title><style>.page{{display:flex;{}}}</style>",self.title,self.page_style.unwrap_or_default().into_html(parent_direction));
    }
}
impl IntoHtml for Document {
    fn into_html(self,parent_direction:ParentDirection)->String {
        let mut out=format!("<!DOCTYPE html><html><head>{}</head><body style=\"margin:0;padding:0\">",self.metadata.into_html(parent_direction));
        for page in self.pages {
            let fmt=page.into_html(parent_direction);
            out.push_str(&fmt);
        }
        out.push_str("</body></html>");
        return out;
    }
}


#[derive(PartialEq,Copy,Clone)]
pub enum ParentDirection {
    Vertical,
    Horizontal,
    None,
}
