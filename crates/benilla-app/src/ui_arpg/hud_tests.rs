//! The ARPG HUD addon (`crate::player::arpg::HUD_LUA`, `arpg_hud.lua`) run in a bare VM: the
//! Forge and the HUD's paint.

use benilla_ui::script::UiScript;

use crate::player::arpg::HUD_LUA;

#[test]
fn the_forge_puts_an_item_in_and_sends_an_orb_and_a_salvage() {
    let script = UiScript::new().expect("VM");
    // The bare VM has no FrameXML: its templates and bag API are stood in for.
    let chunk = format!(
        "if not UIParent then CreateFrame(\"Frame\", \"UIParent\") end\n\
         for _, f in ipairs({{\"GameFontHighlight\", \"GameFontNormalSmall\", \"GameFontNormal\",\n\
         \"GameFontHighlightSmall\", \"GameFontNormalLarge\", \"GameFontDisableSmall\", \"NumberFontNormal\"}}) do\n\
         if not _G[f] then CreateFont(f) end end\n\
         CreateFrame(\"Frame\", \"MainMenuBar\", UIParent)\n\
         CreateFrame(\"Frame\", \"MainMenuBarLeftEndCap\", MainMenuBar)\n\
         for i = 1, 12 do CreateFrame(\"CheckButton\", \"ActionButton\" .. i, MainMenuBar) end\n\
         function UIParent_ManageFramePositions() end\n\
         local stockCreate = CreateFrame\n\
         CreateFrame = function(kind, name, parent, template) return stockCreate(kind, name, parent) end\n\
         function GetContainerItemInfo(bag, slot) if bag == 0 and slot == 3 then return \"Interface\\\\Icons\\\\INV_Sword_04\" end end\n\
         function GetContainerItemLink(bag, slot) return \"[Test Sword]\" end\n\
         SENT = {{}}\n\
         function SetCVar(name, value) SENT[name] = value end\n\
         function GetCVar(name) return SENT[name] end\n\
         {HUD_LUA}\n\
         ArpgPurse_Update({{orbs={{{{id=1,held=2,name=\"Orb of Awakening\",text=\"x\"}},\n\
         {{id=4,held=1,name=\"Fel Orb\",text=\"y\"}}}},shards={{}}}})\n\
         ArpgForge_Toggle()\n\
         assert(ArpgForgeFrame:IsShown(), \"the Forge opens\")\n\
         forgeSelect(0, 3)\n\
         assert(forgeItem and forgeItem.bag == 0 and forgeItem.slot == 3, \"the item is in\")\n\
         ArpgForgeOrb2:Click()\n\
         FIRST = GetCVar(\"arpgCraft\")\n\
         ArpgForgeSalvage:Click()\n\
         ArpgForgeSalvage:Click()\n\
         SECOND = GetCVar(\"arpgCraft\")\n"
    );
    script.run(&chunk).expect("the Forge runs");
    let first = script.eval::<String>("return FIRST").unwrap();
    assert!(
        first.starts_with("4:0:3:0:"),
        "the Fel Orb's craft: {first}"
    );
    let second = script.eval::<String>("return SECOND").unwrap();
    assert!(second.starts_with("8:0:3:0:"), "the salvage: {second}");
}

#[test]
fn the_hud_addon_loads_and_paints() {
    let script = UiScript::new().expect("VM");
    // The addon's own code, then a paint of each part in the same chunk, where its locals are
    // in scope; the bare VM has no player, so the orbs read empty.
    let chunk = format!(
        "if not UIParent then CreateFrame(\"Frame\", \"UIParent\") end\n\
         for _, f in ipairs({{\"GameFontHighlight\", \"GameFontNormalSmall\", \"GameFontNormal\",\n\
         \"GameFontHighlightSmall\"}}) do\n\
         if not _G[f] then CreateFont(f) end end\n\
         CreateFrame(\"Frame\", \"MainMenuBar\", UIParent)\n\
         CreateFrame(\"Frame\", \"MainMenuBarLeftEndCap\", MainMenuBar)\n\
         for i = 1, 12 do CreateFrame(\"CheckButton\", \"ActionButton\" .. i, MainMenuBar) end\n\
         MANAGED = 0\n\
         function UIParent_ManageFramePositions() MANAGED = MANAGED + 1 end\n\
         {HUD_LUA}\n\
         assert(not MainMenuBarLeftEndCap:IsVisible())\n\
         assert(ActionButton12:IsVisible())\n\
         assert(not MainMenuBar:IsMouseEnabled())\n\
         MainMenuBarLeftEndCap:SetParent(MainMenuBar)\nMainMenuBarLeftEndCap:Show()\n\
         UIParent_ManageFramePositions()\n\
         assert(MANAGED == 1 and not MainMenuBarLeftEndCap:IsVisible())\n\
         refresh()\nrefreshOrbs()\n\
         paintOrb(healthOrb, 0.5, HEALTH_COLOR, 50)\n\
         assert(healthOrb.target == 0.5, healthOrb.target)\n\
         for i = 1, 60 do animateOrb(healthOrb, 0.1, i * 0.1) end\n\
         assert(math.abs(healthOrb.shown - 0.5) < 0.01, healthOrb.shown)\n\
         assert(healthOrb.cols[1].wet ~= nil)\n\
         paintOrb(powerOrb, 2, POWER_COLORS[1], 9)\n\
         assert(powerOrb.target == 1)\n\
         ArpgEvent_Update({{state=1,kind=3,title=\"Siege\",objective=\"Hold\",have=1,need=3,\n\
         secs=75,dist=120,dir=\"NE\"}})\n\
         assert(eventBlock:IsShown() and eventBlock.where:GetText() == \"1:15  120 yd NE\",\n\
         eventBlock.where:GetText())\n\
         ArpgBounties_Update({{{{slot=0,kind=2,have=3,need=20,title=\"Westfall\",reward=\"gold\"}}}})\n\
         assert(bountyRows[1]:IsShown() and not bountyRows[2]:IsShown())\n\
         ArpgEvent_Update(nil)\n\
         assert(not eventBlock:IsShown())\n\
         ArpgPurse_Update({{orbs={{}},shards={{}}}})\n"
    );
    script.run(&chunk).expect("the HUD addon runs");
}
