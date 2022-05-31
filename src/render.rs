//use pulldown_cmark::{};
use crate::parser::{
    Document,
    Metadata,
    Page,
    PageStyle,
    Section,
    SectionStyle,
    Size,
    Color,
    PageSize,
    SizedSides,
    Direction,
};


pub trait IntoHtml {
    fn into_html(self)->String;
}
impl IntoHtml for Size {
    fn into_html(self)->String {
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
    fn into_html(self)->String {
        format!("#{:2X}{:2X}{:2X}{:2X}",self.r,self.g,self.b,self.a.unwrap_or(0xff))
    }
}
impl IntoHtml for Direction {
    fn into_html(self)->String {
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
    fn into_html(self)->String {
        use PageSize::*;
        match self {
            PortraitLetter=>format!("width:8.5in;height:11in;"),
            LandscapeLetter=>format!("width:11in;height:8.5in;"),
            Webpage=>format!("width:100%;height:100%;"),
            Custom{width,height}=>format!("width:{};height:{};",width.into_html(),height.into_html()),
        }
    }
}
impl IntoHtml for SectionStyle {
    fn into_html(self)->String {
        let mut out="style=\"".to_string();
        if let Some(width)=self.width {
            let fmt=format!("width:{};",width.into_html());
            out.push_str(&fmt);
        }
        if let Some(height)=self.height {
            let fmt=format!("height:{};",height.into_html());
            out.push_str(&fmt);
        }
        if let Some(align)=self.align {
            let fmt=format!("align:{};",align.into_html());
            out.push_str(&fmt);
        }
        if let Some(font)=self.font {
            let fmt=format!("font:{};",font);
            out.push_str(&fmt);
        }
        if let Some(font_size)=self.font_size {
            let fmt=format!("font-size:{};",font_size.into_html());
            out.push_str(&fmt);
        }
        if let Some(text_color)=self.text_color {
            let fmt=format!("color:{};",text_color.into_html());
            out.push_str(&fmt);
        }
        if let Some(background_color)=self.background_color {
            let fmt=format!("background-color:{};",background_color.into_html());
            out.push_str(&fmt);
        }
        out.push('"');
        return out;
    }
}
impl IntoHtml for Section {
    fn into_html(self)->String {
        let mut out=format!("<div {}>",self.style.unwrap_or_default().into_html());
        // TODO: cmark parsing
        for s in self.content {
            out.push_str("<p>");
            out.push_str(&s);
            out.push_str("</p>");
        }
        out.push_str("</div>");
        return out;
    }
}
impl IntoHtml for SizedSides {
    fn into_html(self)->String {
        use SizedSides::*;
        match self {
            All(size)=>{
                return format!("NAME:{};",size.into_html());
            },
            Individual{left,right,top,bottom}=>{
                let mut out=String::new();
                if let Some(size)=left {
                    let fmt=format!("NAME-left:{};",size.into_html());
                    out.push_str(&fmt);
                }
                if let Some(size)=right {
                    let fmt=format!("NAME-right:{};",size.into_html());
                    out.push_str(&fmt);
                }
                if let Some(size)=top {
                    let fmt=format!("NAME-top:{};",size.into_html());
                    out.push_str(&fmt);
                }
                if let Some(size)=bottom {
                    let fmt=format!("NAME-bottom:{};",size.into_html());
                    out.push_str(&fmt);
                }
                return out;
            },
        }
    }
}
impl IntoHtml for PageStyle {
    fn into_html(self)->String {
        let mut out=String::new();
        if let Some(page_size)=self.page_size {
            let fmt=page_size.into_html();
            out.push_str(&fmt);
        }
        if let Some(text_color)=self.text_color {
            let fmt=format!("color:{};",text_color.into_html());
            out.push_str(&fmt);
        }
        if let Some(background_color)=self.background_color {
            let fmt=format!("background-color:{};",background_color.into_html());
            out.push_str(&fmt);
        }
        if let Some(margin)=self.margin {
            let fmt=margin.into_html().replace("NAME","margin");
            out.push_str(&fmt);
        }
        if let Some(padding)=self.padding {
            let fmt=padding.into_html().replace("NAME","padding");
            out.push_str(&fmt);
        }
        return out;
    }
}
impl IntoHtml for Page {
    fn into_html(self)->String {
        let mut out=format!("<div style=\"{}\" class=\"page\">",self.style.unwrap_or_default().into_html());
        for section in self.sections {
            let fmt=section.into_html();
            out.push_str(&fmt);
        }
        out.push_str("</div>");
        return out;
    }
}
impl IntoHtml for Metadata {
    fn into_html(self)->String {
        let mut out=format!("<div><style>{}</style>
        return out;
    }
}
