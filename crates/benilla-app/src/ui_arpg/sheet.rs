//! Fork-only, not 1.12.1: the feed half of the ARPG server's character sheet
//! (`crate::player::arpg::sheet`), handed to the HUD addon as `ArpgSheet_Update`.

use bevy::prelude::*;

use benilla_protocol::messages::ArpgSheet;
use benilla_ui::script::UiScript;

use crate::player::arpg::sheet::ArpgSheetState;

use super::tree::lua_str;

/// The Lua chunk that hands `sheet` to the addon; it returns whether the addon took it.
fn sheet_chunk(sheet: &ArpgSheet) -> String {
    let attributes: Vec<String> = sheet.attributes.iter().map(|a| lua_str(a)).collect();
    let skills: Vec<String> = sheet
        .skills
        .iter()
        .map(|(id, text)| format!("[{id}]={}", lua_str(text)))
        .collect();
    format!(
        "if not ArpgSheet_Update then return false end\n\
         ArpgSheet_Update({{attributes={{{}}},skills={{{}}},summary={}}})\n\
         return true",
        attributes.join(","),
        skills.join(","),
        lua_str(&sheet.summary),
    )
}

/// Hand the addon the newest sheet, once per sheet and VM (a reloaded UI is a new VM).
pub(crate) fn push_sheet(script: Option<NonSendMut<UiScript>>, mut state: ResMut<ArpgSheetState>) {
    let Some(script) = script else {
        return;
    };
    let mark = (script.session(), state.generation());
    if state.pushed() == Some(mark) {
        return;
    }
    let Some(sheet) = state.sheet() else {
        return;
    };
    match script.eval::<bool>(&sheet_chunk(sheet)) {
        Ok(true) => state.set_pushed(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the HUD refused the character sheet: {e}");
            state.set_pushed(mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sheet_chunk_hands_the_addon_its_attributes_skills_and_summary() {
        let sheet = ArpgSheet {
            attributes: vec!["+40 attack power.\n|cffffd200Melee area +4%|r".into()],
            skills: vec![(2, "Tags: Holy".into())],
            summary: "Movement speed \"+10%\"".into(),
        };
        let script = UiScript::new().expect("a UI VM");
        script
            .run("function ArpgSheet_Update(t) GOT = t end")
            .expect("the stub loads");
        assert!(script.eval::<bool>(&sheet_chunk(&sheet)).unwrap());
        assert_eq!(
            script.eval::<String>("return GOT.attributes[1]").unwrap(),
            "+40 attack power.\n|cffffd200Melee area +4%|r"
        );
        assert_eq!(
            script.eval::<String>("return GOT.skills[2]").unwrap(),
            "Tags: Holy"
        );
        assert_eq!(
            script.eval::<String>("return GOT.summary").unwrap(),
            "Movement speed \"+10%\""
        );
    }
}
