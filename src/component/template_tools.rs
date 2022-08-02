pub struct TemplateHint {
    hint_infos:Vec<TemplateHintInfo>
}



impl TemplateHint {

    pub fn new() -> Self {
        Self {
            hint_infos:Vec::new()
        }
    }

    pub fn add(&mut self,hint_info:TemplateHintInfo) {
        self.hint_infos.push(hint_info);
    }
    
    pub fn ui(&self,ui: &mut egui::Ui,edit_pos:Option<(usize,usize)>,code:&mut String) -> bool{
        ui.columns(3, |ui|{
                        let l = self.hint_infos.len();
                        let mut results:Vec<bool> = Vec::new();
                        for i in 0..l {
                            let c = i%3;
                            let r = self.hint_infos[i].ui(&mut ui[c], edit_pos, code);
                            results.push(r);
                            if c==2 {
                                ui[c].end_row();
                            }
                        }
                        results.into_iter().any(|i|i)
                })

        // egui::Grid::new("id_source_adsfasdfasdfas")
        //     .num_columns(3)
        //     .min_col_width(80.)
        //     .show(ui, |ui|{
        //         let l = self.hint_infos.len();
        //         let mut results:Vec<bool> = Vec::new();
        //         for i in 0..l {
        //             let r = self.hint_infos[i].ui(ui, edit_pos, code);
        //             results.push(r);
        //             if i%3==2 {
        //                 ui.end_row();
        //             }
        //         }
        //         results.into_iter().any(|i|i)
        //     }).inner

        // let one = self.hint_infos.iter()
        //         .enumerate()
        //         .filter(|(i,h)| i%3==0)
        //         .map(|x|x.1);
        // let two = self.hint_infos.iter()
        //         .enumerate()
        //         .filter(|(i,h)| i%3==1)
        //         .map(|x|x.1);
        // let thr = self.hint_infos.iter()
        //         .enumerate()
        //         .filter(|(i,h)| i%3==2)
        //         .map(|x|x.1);
        // one.zip(two).zip(thr)
        // .map(|((one,two),thr)|{
        //     let row = ui.horizontal(|ui|{
        //         [one.ui(ui, edit_pos, code),
        //         two.ui(ui, edit_pos, code),
        //         thr.ui(ui, edit_pos, code)]
        //     }).inner;
        //     row
        // })
        // .flatten()
        // .any(|sub_resp|sub_resp)

        // .map(|hint_info|{
        //     hint_info.ui(ui, edit_pos, code)
        // })
        // .any(|sub_resp|sub_resp)
    }
}

pub struct TemplateHintInfo {
    button:String,
    hint:String,
    text:String,
}

impl TemplateHintInfo {
    
    pub fn new(button:String,hint:String,text:String) -> Self {
        Self { button, hint , text }
    }

    pub fn ui(&self,ui: &mut egui::Ui,edit_pos:Option<(usize,usize)>,code:&mut String) -> bool{
        if ui.button(&self.button).on_hover_text(self.hint.clone()).clicked() {
            if let Some((start,end)) = edit_pos {
                if start == end {
                    if end < code.len() {
                        let start:String = code.chars().take(end).collect();
                        let end:String = code.chars().skip(end).collect();
                        *code = start + &self.text + &end; 
                    } else {
                        code.push_str(&self.text);
                    }
                } else {
                    code.replace_range(start..end,&self.text);
                }
            } else {
                code.push_str(&self.text);
            }
            return true;
        }
        false
    }
}