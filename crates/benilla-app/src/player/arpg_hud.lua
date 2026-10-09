-- ArpgHud: the benilla ARPG client's hover health bar, written into AddOns by the client while the
-- ARPG view is on (crates/benilla-app/src/player/arpg.rs). There is no target in the ARPG view, so
-- the enemy under the cursor ("mouseover") gets a bar at the top of the screen instead. The
-- player's health and power show as two orbs at the bottom corners.

local WIDTH, HEIGHT = 360, 18

local bar = CreateFrame("StatusBar", "ArpgHudBar", UIParent)
bar:SetWidth(WIDTH)
bar:SetHeight(HEIGHT)
bar:SetPoint("TOP", UIParent, "TOP", 0, -48)
bar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
bar:SetStatusBarColor(0.75, 0.08, 0.08)
bar:SetMinMaxValues(0, 1)
bar:SetValue(1)
bar:SetFrameStrata("HIGH")

local back = bar:CreateTexture(nil, "BACKGROUND")
back:SetAllPoints(bar)
back:SetTexture(0, 0, 0, 0.65)

local name = bar:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
name:SetPoint("CENTER", bar, "CENTER", 0, 0)

local rank = bar:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
rank:SetPoint("BOTTOM", bar, "TOP", 0, 3)

bar:Hide()

-- Gold for elites and bosses, silver for rares, as the stock target frame's dragon.
local RANKS = {
    worldboss = { "Boss", 1.0, 0.5, 0.0 },
    elite = { "Elite", 1.0, 0.82, 0.0 },
    rareelite = { "Rare Elite", 0.8, 0.8, 1.0 },
    rare = { "Rare", 0.8, 0.8, 1.0 },
}

local function refresh()
    if not UnitExists("mouseover") or not UnitCanAttack("player", "mouseover")
        or UnitIsDead("mouseover") then
        bar:Hide()
        return
    end
    local health, most = UnitHealth("mouseover"), UnitHealthMax("mouseover")
    if not most or most <= 0 then
        most = 1
    end
    bar:SetMinMaxValues(0, most)
    bar:SetValue(health or 0)
    name:SetText(UnitName("mouseover"))

    local level = UnitLevel("mouseover")
    local label = (level and level > 0) and ("Level " .. level) or "Level ??"
    local special = RANKS[UnitClassification("mouseover") or "normal"]
    if special then
        rank:SetText(label .. "  " .. special[1])
        rank:SetTextColor(special[2], special[3], special[4])
    else
        rank:SetText(label)
        rank:SetTextColor(1.0, 0.82, 0.0)
    end
    bar:Show()
end

-- A few refreshes a second follow the hover and the health without an event per unit.
local driver = CreateFrame("Frame")
local since = 0
driver:SetScript("OnUpdate", function()
    since = since + (arg1 or 0)
    if since >= 0.05 then
        since = 0
        refresh()
    end
end)
driver:RegisterEvent("UPDATE_MOUSEOVER_UNIT")
driver:SetScript("OnEvent", refresh)

-- The health and power orbs, Diablo's globes, at the bottom corners of the screen. Each is a
-- stack of horizontal slices cut to a circle; the slices under the fill line take the orb's colour,
-- shaded darker toward the rim, and those above it stay dark glass. The power orb takes the
-- player's power colour: blue mana, red rage, yellow energy.

local ORB_RADIUS, ORB_SLICES, ORB_INSET = 56, 48, 18
local RIM = 3

local POWER_COLORS = {
    [0] = { 0.12, 0.30, 0.95 }, -- mana
    [1] = { 0.85, 0.10, 0.08 }, -- rage
    [2] = { 0.95, 0.55, 0.15 }, -- focus
    [3] = { 0.95, 0.85, 0.15 }, -- energy
}
local HEALTH_COLOR = { 0.80, 0.06, 0.06 }

local function makeOrb(frameName, corner, x)
    local size = ORB_RADIUS * 2
    local orb = CreateFrame("Frame", frameName, UIParent)
    orb:SetWidth(size + RIM * 2)
    orb:SetHeight(size + RIM * 2)
    orb:SetPoint(corner, UIParent, corner, x, ORB_INSET)
    orb:SetFrameStrata("MEDIUM")

    local step = size / ORB_SLICES
    orb.slices = {}
    orb.shade = {}
    for i = 1, ORB_SLICES do
        -- The slice's centre height over the orb's centre, and its half-width on the circle.
        local y = (i - 0.5) * step - ORB_RADIUS
        local half = math.sqrt(math.max(ORB_RADIUS * ORB_RADIUS - y * y, 0))
        -- The rim: a dark band either side of the glass.
        local rhalf = math.sqrt(math.max((ORB_RADIUS + RIM) * (ORB_RADIUS + RIM) - y * y, 0))
        local rim = orb:CreateTexture(nil, "BACKGROUND")
        rim:SetTexture(0.05, 0.04, 0.03, 0.9)
        rim:SetWidth(math.max(rhalf * 2, 1))
        rim:SetHeight(step + 0.5)
        rim:SetPoint("BOTTOM", orb, "BOTTOM", 0, RIM + (i - 1) * step)

        local slice = orb:CreateTexture(nil, "ARTWORK")
        slice:SetWidth(math.max(half * 2, 1))
        slice:SetHeight(step + 0.5)
        slice:SetPoint("BOTTOM", orb, "BOTTOM", 0, RIM + (i - 1) * step)
        orb.slices[i] = slice
        -- Lighter in the upper middle, darker at the bottom: a little roundness.
        orb.shade[i] = 0.55 + 0.45 * math.sqrt(math.sin(math.pi * (i - 0.5) / ORB_SLICES))
            * (0.75 + 0.25 * (i / ORB_SLICES))
    end
    -- The glare: a pale streak on the glass's upper left.
    local glare = orb:CreateTexture(nil, "OVERLAY")
    glare:SetTexture(1, 1, 1, 0.12)
    glare:SetWidth(ORB_RADIUS * 0.5)
    glare:SetHeight(ORB_RADIUS * 0.22)
    glare:SetPoint("CENTER", orb, "CENTER", -ORB_RADIUS * 0.3, ORB_RADIUS * 0.55)

    orb.text = orb:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    orb.text:SetPoint("CENTER", orb, "CENTER", 0, 0)
    orb.filled = -1
    orb.r, orb.g, orb.b = -1, -1, -1
    return orb
end

local healthOrb = makeOrb("ArpgHudHealthOrb", "BOTTOMLEFT", ORB_INSET)
local powerOrb = makeOrb("ArpgHudPowerOrb", "BOTTOMRIGHT", -ORB_INSET)

-- Paint the orb `fraction` full in this colour; only a change repaints the slices.
local function paintOrb(orb, fraction, color, value)
    local filled = math.floor(math.max(math.min(fraction, 1), 0) * ORB_SLICES + 0.5)
    local r, g, b = color[1], color[2], color[3]
    if filled ~= orb.filled or r ~= orb.r or g ~= orb.g or b ~= orb.b then
        for i = 1, ORB_SLICES do
            local k = orb.shade[i]
            if i <= filled then
                orb.slices[i]:SetTexture(r * k, g * k, b * k, 0.95)
            else
                orb.slices[i]:SetTexture(0.06 * k, 0.05 * k, 0.07 * k, 0.85)
            end
        end
        orb.filled, orb.r, orb.g, orb.b = filled, r, g, b
    end
    orb.text:SetText(value)
end

local function refreshOrbs()
    local health, most = UnitHealth("player") or 0, UnitHealthMax("player") or 0
    paintOrb(healthOrb, most > 0 and health / most or 0, HEALTH_COLOR, health)
    local power, top = UnitMana("player") or 0, UnitManaMax("player") or 0
    local color = POWER_COLORS[UnitPowerType("player") or 0] or POWER_COLORS[0]
    paintOrb(powerOrb, top > 0 and power / top or 0, color, power)
end

local orbDriver = CreateFrame("Frame")
local orbSince = 0
orbDriver:SetScript("OnUpdate", function()
    orbSince = orbSince + (arg1 or 0)
    if orbSince >= 0.05 then
        orbSince = 0
        refreshOrbs()
    end
end)

-- ── The options window's ARPG page ──
-- benilla's options window (BenillaOptionsFrame) gains an "ARPG View" category under its own
-- header, built from the window's own row templates, so its rows store, refresh, search and reset
-- like every other page. Each slider writes its setting at once, and the client reads the settings
-- every frame, so the camera and the cut move as the thumb does. A bare VM has no window: skip.

ARPG_TOOLTIP_CAMERA_YAW = "Turns the camera around your character, in degrees."
ARPG_TOOLTIP_CAMERA_PITCH = "How steeply the camera looks down, in degrees: 90 is straight down."
ARPG_TOOLTIP_CAMERA_DISTANCE = "How far the camera sits from your character, in yards."
ARPG_TOOLTIP_CUT_HEIGHT = "Indoors and in caves, walls and ceilings more than this many yards "
    .. "above your feet are cut away. Lower shows more of the room; higher keeps more walls."
ARPG_TOOLTIP_CUT_RADIUS = "Indoors and in caves, how far from your character the cut reaches, "
    .. "in yards. Geometry farther out keeps its tops."
ARPG_TOOLTIP_LOOT_FILTER = "Which items on the ground get a name label. Hidden ones still "
    .. "glow and can be picked up, and holding Alt shows every label. Gold always shows."
ARPG_TOOLTIP_DEV_LOOT = "For testing: Drop Test Loot asks the server for a corpse at your feet "
    .. "holding this many items of this quality around this item level. Needs Arpg.DevTools = 1 "
    .. "in the server's mangosd.conf."
ARPG_TOOLTIP_SEE_THROUGH = "Outdoors, roofs, awnings and trees between your character and the "
    .. "camera turn see-through."

-- `cvar`, the label, the tooltip global, then a slider's min, max, step and readout format (or,
-- with no format, a named stop per step); a row with no range is a checkbox.
local ARPG_OPTION_ROWS = {
    { "RowSeeThrough", "arpgSeeThrough", "See-Through Outdoors", "ARPG_TOOLTIP_SEE_THROUGH" },
    { "RowCameraDistance", "arpgCameraDistance", "Camera Distance", "ARPG_TOOLTIP_CAMERA_DISTANCE",
        5, 50, 1, "%d yd" },
    { "RowCameraPitch", "arpgCameraPitch", "Camera Pitch", "ARPG_TOOLTIP_CAMERA_PITCH",
        20, 89, 1, "%d deg" },
    { "RowCameraYaw", "arpgCameraYaw", "Camera Yaw", "ARPG_TOOLTIP_CAMERA_YAW",
        0, 360, 5, "%d deg" },
    { "RowCutHeight", "arpgCutHeight", "Cutaway Height", "ARPG_TOOLTIP_CUT_HEIGHT",
        1.5, 10, 0.1, "%.1f yd" },
    { "RowCutRadius", "arpgCutRadius", "Cutaway Radius", "ARPG_TOOLTIP_CUT_RADIUS",
        5, 120, 1, "%d yd" },
    { "RowLootFilter", "arpgLootFilter", "Loot Labels", "ARPG_TOOLTIP_LOOT_FILTER",
        0, 3, 1, nil, { "All", "No Grey", "Green and Better", "Blue and Better" } },
    -- Developer: test loot, which a server with Arpg.DevTools = 1 drops at your feet.
    { "RowDevLootQuality", "arpgDevLootQuality", "Test Loot Quality", "ARPG_TOOLTIP_DEV_LOOT",
        0, 6, 1, nil, { "Mixed", "Grey", "White", "Green", "Blue", "Purple", "Orange" } },
    { "RowDevLootCount", "arpgDevLootCount", "Test Loot Count", "ARPG_TOOLTIP_DEV_LOOT",
        1, 16, 1, "%d" },
    { "RowDevLootLevel", "arpgDevLootLevel", "Test Loot Level", "ARPG_TOOLTIP_DEV_LOOT",
        0, 60, 1, function(v)
            if v < 0.5 then return "Mine" end
            return string.format("%d", math.floor(v + 0.5))
        end },
}

local function addArpgOptionsPage()
    if not BenillaOptionsFrame or not BENILLA_OPTIONS_PAGE_ROWS
        or not BENILLA_OPTIONS_CATEGORY_KEYS or BENILLA_OPTIONS_PAGE_ROWS.Arpg then
        return
    end
    local list = BenillaOptionsFrameCategoryList
    local header = CreateFrame("Frame", "BenillaOptionsFrameCategoryListHeaderArpg", list,
        "BenillaOptionsCategoryHeaderTemplate")
    header:SetPoint("TOPLEFT", "BenillaOptionsFrameCategoryListRowAudio", "BOTTOMLEFT", 0, -22)
    getglobal(header:GetName() .. "Label"):SetText("ARPG")
    local entry = CreateFrame("Button", "BenillaOptionsFrameCategoryListRowArpg", list,
        "BenillaOptionsCategoryRowTemplate")
    entry:SetPoint("TOPLEFT", header, "BOTTOMLEFT", 0, -2)
    BenillaOptionsCategoryRow_OnLoad(entry, "Arpg", "ARPG View")

    local body = CreateFrame("Frame", "BenillaOptionsFrameContainerBodyArpg",
        BenillaOptionsFrameContainerBody)
    body:SetPoint("TOPLEFT", BenillaOptionsFrameContainerBody, "TOPLEFT", 0, 0)
    body:SetPoint("BOTTOMRIGHT", BenillaOptionsFrameContainerBody, "BOTTOMRIGHT", 0, 0)
    body:Hide()
    -- A search's heading over our matches, as the window has one per page.
    local head = CreateFrame("Button", "BenillaOptionsFrameContainerBodySearchHeadArpg",
        BenillaOptionsFrameContainerBody, "BenillaOptionsSearchHeadTemplate")
    head.categoryKey = "Arpg"
    getglobal(head:GetName() .. "Label"):SetText("ARPG View")

    local keys, prev = {}, nil
    for _, spec in ipairs(ARPG_OPTION_ROWS) do
        local key, cvar, label, tip, minv, maxv, step, fmt, stops =
            spec[1], spec[2], spec[3], spec[4], spec[5], spec[6], spec[7], spec[8], spec[9]
        local template = "BenillaOptionsCheckboxRowTemplate"
        if minv then template = "BenillaOptionsSliderRowTemplate" end
        local row = CreateFrame("Frame", body:GetName() .. key, body, template)
        if prev then
            row:SetPoint("TOPLEFT", prev, "BOTTOMLEFT", 0, -9)
            row:SetPoint("TOPRIGHT", prev, "BOTTOMRIGHT", 0, -9)
        else
            row:SetPoint("TOPLEFT", body, "TOPLEFT", 10, -12)
            row:SetPoint("TOPRIGHT", body, "TOPRIGHT", -20, -12)
        end
        BenillaOptionsRow_OnLoad(row, cvar, label, tip)
        if minv then
            BenillaOptionsSliderRow_Setup(row, minv, maxv, step, "arpg", stops)
            row.arpgFmt = fmt
        end
        table.insert(keys, key)
        prev = row
    end
    -- Drop Test Loot: a child of the last row, so a search hides it with that row. The client
    -- sends the request when the counter setting moves (arpgDevLootDrop, never saved).
    local drop = CreateFrame("Button", body:GetName() .. "DevLootDrop", prev,
        "BenillaOptionsRedButtonTemplate")
    drop:SetWidth(140)
    drop:SetPoint("TOPLEFT", prev, "BOTTOMLEFT", 37, -10)
    drop:SetText("Drop Test Loot")
    drop:SetScript("OnClick", function()
        PlaySound("igMainMenuOptionCheckBoxOn")
        SetCVar("arpgDevLootDrop", tostring((tonumber(GetCVar("arpgDevLootDrop")) or 0) + 1))
    end)

    BENILLA_OPTIONS_PAGE_ROWS.Arpg = keys
    table.insert(BENILLA_OPTIONS_CATEGORY_KEYS, "Arpg")

    -- The window's readouts know only its own formats; ours carry a printf pattern.
    local formatValue = BenillaOptionsRow_FormatValue
    BenillaOptionsRow_FormatValue = function(row, value)
        if type(row.arpgFmt) == "function" then return row.arpgFmt(value) end
        if row.arpgFmt then
            -- Whole-number readouts round rather than truncate.
            if string.find(row.arpgFmt, "%%d") then value = math.floor(value + 0.5) end
            return string.format(row.arpgFmt, value)
        end
        return formatValue(row, value)
    end
end

addArpgOptionsPage()

-- ── The ARPG skill tree window ──
-- The server owns the tree (Arpg/ArpgTree.h): the client hands each layout it sends to
-- ArpgTree_Update. The talent key and the talent micro button open this window in place of the
-- talent frame; a click spends a point, Respec refunds them all (free, out of combat). Requests go
-- back through the session-only setting arpgTreeAction, as a number.

local TREE_GATES = { 0, 5, 10, 20 }
local TREE_KINDS = {
    [0] = { "Passive", 0.55, 0.62, 0.75 },
    [1] = { "Skill", 1.0, 0.82, 0.0 },
    [2] = { "Modifier", 0.902, 0.8, 0.502 },
    [3] = { "Keystone", 1.0, 0.5, 0.0 },
}
local TREE_NODE, TREE_COL, TREE_ROW = 40, 64, 84
local treeData, treeFrame, treeNonce = nil, nil, 0

-- kind 1 spend, 2 respec, 3 query: kind * 100000 + node * 100 + a nonce under 100.
local function treeAsk(kind, node)
    treeNonce = treeNonce + 1
    if treeNonce >= 100 then treeNonce = 1 end
    SetCVar("arpgTreeAction", tostring(kind * 100000 + (node or 0) * 100 + treeNonce))
end

local function treeSpentIn(branch)
    local spent = 0
    for _, n in ipairs(treeData.nodes) do
        if n.branch == branch then spent = spent + n.rank end
    end
    return spent
end

-- Why a point cannot go into `n` now, or nil when it can.
local function treeBlocked(n)
    if n.rank >= n.max then return "Fully learned" end
    if treeData.spent >= treeData.total then return "No points left" end
    local gate = TREE_GATES[n.tier] or 0
    if treeSpentIn(n.branch) < gate then
        return "Requires " .. gate .. " points in " .. (treeData.branches[n.branch + 1] or "this branch")
    end
    return nil
end

local function treeShowTip(button)
    local n = button.node
    if not n then return end
    local kind = TREE_KINDS[n.kind] or TREE_KINDS[0]
    GameTooltip:SetOwner(button, "ANCHOR_RIGHT")
    GameTooltip:SetText(n.name, 1, 1, 1)
    GameTooltip:AddLine(kind[1] .. "   Rank " .. n.rank .. "/" .. n.max, kind[2], kind[3], kind[4])
    GameTooltip:AddLine(n.text, 1, 0.82, 0, 1)
    local blocked = treeBlocked(n)
    if blocked then
        GameTooltip:AddLine(blocked, 1, 0.13, 0.13)
    else
        GameTooltip:AddLine("Click to learn", 0, 1, 0)
    end
    GameTooltip:Show()
end

local function treeMakeButton(parent, name)
    local b = CreateFrame("Button", name, parent)
    b:SetWidth(TREE_NODE)
    b:SetHeight(TREE_NODE)
    b.border = b:CreateTexture(nil, "BACKGROUND")
    b.border:SetPoint("TOPLEFT", b, "TOPLEFT", -3, 3)
    b.border:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", 3, -3)
    b.icon = b:CreateTexture(nil, "ARTWORK")
    b.icon:SetAllPoints(b)
    b.rank = b:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    b.rank:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", 2, -2)
    b:SetScript("OnEnter", function() treeShowTip(this) end)
    b:SetScript("OnLeave", function() GameTooltip:Hide() end)
    b:SetScript("OnClick", function()
        if this.node and not treeBlocked(this.node) then
            treeAsk(1, this.node.id)
        end
    end)
    return b
end

local function treeBuild()
    if treeFrame then return treeFrame end
    local f = CreateFrame("Frame", "ArpgTreeFrame", UIParent)
    f:SetWidth(3 * 230 + 40)
    f:SetHeight(4 * TREE_ROW + 110)
    f:SetPoint("CENTER", UIParent, "CENTER", 0, 40)
    f:SetFrameStrata("DIALOG")
    f:EnableMouse(true)
    f:Hide()
    local bg = f:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(f)
    bg:SetTexture(0.04, 0.035, 0.05, 0.94)
    local title = f:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    title:SetPoint("TOP", f, "TOP", 0, -12)
    title:SetText("Skill Tree")
    f.points = f:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    f.points:SetPoint("TOPLEFT", f, "TOPLEFT", 20, -16)
    local close = CreateFrame("Button", "ArpgTreeFrameClose", f, "UIPanelCloseButton")
    close:SetPoint("TOPRIGHT", f, "TOPRIGHT", -4, -4)
    local respec = CreateFrame("Button", "ArpgTreeFrameRespec", f, "UIPanelButtonTemplate")
    respec:SetWidth(90)
    respec:SetHeight(22)
    respec:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -16, 12)
    respec:SetText("Respec")
    respec:SetScript("OnClick", function() treeAsk(2) end)
    f.columns, f.buttons = {}, {}
    for b = 1, 3 do
        local col = CreateFrame("Frame", nil, f)
        col:SetWidth(220)
        col:SetHeight(4 * TREE_ROW + 30)
        col:SetPoint("TOPLEFT", f, "TOPLEFT", 20 + (b - 1) * 230, -44)
        local shade = col:CreateTexture(nil, "BACKGROUND")
        shade:SetAllPoints(col)
        shade:SetTexture(1, 1, 1, 0.04)
        col.label = col:CreateFontString(nil, "OVERLAY", "GameFontNormal")
        col.label:SetPoint("TOP", col, "TOP", 0, -6)
        f.columns[b] = col
    end
    if UISpecialFrames then tinsert(UISpecialFrames, "ArpgTreeFrame") end
    treeFrame = f
    return f
end

local function treeRedraw()
    if not treeData then return end
    local f = treeBuild()
    f.points:SetText("Points: " .. (treeData.total - treeData.spent) .. " of " .. treeData.total .. " left")
    for b = 1, 3 do
        local col = f.columns[b]
        local name = treeData.branches[b]
        if name then
            col.label:SetText(name .. "  (" .. treeSpentIn(b - 1) .. ")")
            col:Show()
        else
            col:Hide()
        end
    end
    for _, button in pairs(f.buttons) do button:Hide() end
    for _, n in ipairs(treeData.nodes) do
        local col = f.columns[n.branch + 1]
        if col then
            local button = f.buttons[n.id]
            if not button then
                button = treeMakeButton(col, "ArpgTreeNode" .. n.id)
                f.buttons[n.id] = button
            end
            button.node = n
            button:ClearAllPoints()
            button:SetPoint("TOPLEFT", col, "TOPLEFT", 22 + n.column * TREE_COL, -30 - (n.tier - 1) * TREE_ROW)
            button.icon:SetTexture(n.icon ~= "" and n.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
            local kind = TREE_KINDS[n.kind] or TREE_KINDS[0]
            local blocked = treeBlocked(n)
            local open = n.rank > 0 or not blocked or blocked == "Fully learned"
            if open then
                button.icon:SetVertexColor(1, 1, 1)
                button.border:SetTexture(kind[2], kind[3], kind[4], 0.9)
            else
                button.icon:SetVertexColor(0.35, 0.35, 0.35)
                button.border:SetTexture(0.25, 0.25, 0.25, 0.9)
            end
            button.rank:SetText(n.rank .. "/" .. n.max)
            if n.rank >= n.max then
                button.rank:SetTextColor(1, 0.82, 0)
            elseif n.rank > 0 then
                button.rank:SetTextColor(0, 1, 0)
            else
                button.rank:SetTextColor(0.8, 0.8, 0.8)
            end
            button:Show()
        end
    end
end

function ArpgTree_Update(tree)
    treeData = tree
    if treeFrame and treeFrame:IsVisible() then treeRedraw() end
end

function ArpgTree_Toggle()
    if not treeData then
        treeAsk(3)
        DEFAULT_CHAT_FRAME:AddMessage("The skill tree has not arrived from the server yet.")
        return
    end
    local f = treeBuild()
    if f:IsVisible() then
        f:Hide()
    else
        treeRedraw()
        f:Show()
    end
end

-- The talent key and the talent micro button open the tree instead.
ToggleTalentFrame = ArpgTree_Toggle

-- A level gives a point: ask for the fresh count.
local treeEvents = CreateFrame("Frame")
treeEvents:RegisterEvent("PLAYER_LEVEL_UP")
treeEvents:SetScript("OnEvent", function() treeAsk(3) end)
