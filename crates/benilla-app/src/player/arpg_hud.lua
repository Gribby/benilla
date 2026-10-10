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

-- A champion's or a rare's affixes, under the bar.
local affix = bar:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
affix:SetPoint("TOP", bar, "BOTTOM", 0, -3)
affix:SetTextColor(0.85, 0.85, 0.85)

bar:Hide()

-- The champion or rare under the cursor, which the client sets as the hover moves:
-- ArpgHud_Champion(tier, name, affixes), tier 1 a champion, 2 a rare; ArpgHud_Champion(0) clears.
local champ = nil
function ArpgHud_Champion(tier, champName, affixes)
    if not tier or tier <= 0 then
        champ = nil
    else
        champ = { tier = tier, name = champName or "", affixes = affixes or "" }
    end
end

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
    local level = UnitLevel("mouseover")
    local label = (level and level > 0) and ("Level " .. level) or "Level ??"
    local special = RANKS[UnitClassification("mouseover") or "normal"]
    if champ then
        -- Champions blue, rares yellow, as their rings.
        local r, g, b, what = 0.30, 0.55, 1.0, "Champion"
        if champ.tier >= 2 then
            r, g, b, what = 1.0, 0.85, 0.25, "Rare"
        end
        local shown = champ.name
        if shown == "" then shown = UnitName("mouseover") end
        name:SetText(shown)
        name:SetTextColor(r, g, b)
        rank:SetText(label .. "  " .. what)
        rank:SetTextColor(r, g, b)
        affix:SetText(champ.affixes)
        affix:Show()
        bar:Show()
        return
    end
    name:SetText(UnitName("mouseover"))
    name:SetTextColor(1.0, 1.0, 1.0)
    affix:Hide()
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

-- The flask (Q) and the dodge roll (Space): the server's charges and cooldown, which the client
-- passes on through ArpgHud_Status. The flask's charges stand as vials beside the health orb, the
-- next one filling as kills come in; the roll's bar beside the power orb fills back up after a
-- roll.
local VIAL_W, VIAL_H, VIAL_GAP = 12, 34, 5
local vials = {}
local function makeVial(i)
    local v = CreateFrame("Frame", nil, UIParent)
    v:SetWidth(VIAL_W)
    v:SetHeight(VIAL_H)
    v:SetPoint("BOTTOMLEFT", healthOrb, "BOTTOMRIGHT", 6 + (i - 1) * (VIAL_W + VIAL_GAP), 4)
    v:SetFrameStrata("MEDIUM")
    local back = v:CreateTexture(nil, "BACKGROUND")
    back:SetAllPoints(v)
    back:SetTexture(0.05, 0.04, 0.03, 0.85)
    v.fill = v:CreateTexture(nil, "ARTWORK")
    v.fill:SetPoint("BOTTOMLEFT", v, "BOTTOMLEFT", 2, 2)
    v.fill:SetWidth(VIAL_W - 4)
    v.fill:SetHeight(1)
    v:Hide()
    return v
end

local flaskLabel = UIParent:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
flaskLabel:SetPoint("BOTTOMLEFT", healthOrb, "BOTTOMRIGHT", 6, VIAL_H + 8)
flaskLabel:SetText("Q")
flaskLabel:Hide()

local dodgeBar = CreateFrame("StatusBar", nil, UIParent)
dodgeBar:SetWidth(70)
dodgeBar:SetHeight(8)
dodgeBar:SetPoint("BOTTOMRIGHT", powerOrb, "BOTTOMLEFT", -8, 8)
dodgeBar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
dodgeBar:SetStatusBarColor(0.85, 0.8, 0.55)
dodgeBar:SetMinMaxValues(0, 1)
dodgeBar:SetValue(1)
dodgeBar:SetFrameStrata("MEDIUM")
local dodgeBack = dodgeBar:CreateTexture(nil, "BACKGROUND")
dodgeBack:SetAllPoints(dodgeBar)
dodgeBack:SetTexture(0, 0, 0, 0.7)
local dodgeLabel = dodgeBar:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
dodgeLabel:SetPoint("BOTTOM", dodgeBar, "TOP", 0, 2)
dodgeLabel:SetText("Space: Roll")
dodgeBar:Hide()

local flask = { charges = 0, max = 0, progress = 0 }
local dodgeReadyAt, dodgeCooldown = 0, 2.5

function ArpgHud_Status(charges, max, progress, dodgeMs, cooldownMs)
    flask.charges, flask.max, flask.progress = charges or 0, max or 0, progress or 0
    dodgeReadyAt = GetTime() + (dodgeMs or 0) / 1000
    if cooldownMs and cooldownMs > 0 then
        dodgeCooldown = cooldownMs / 1000
    end
    for i = 1, flask.max do
        local v = vials[i] or makeVial(i)
        vials[i] = v
        local h = VIAL_H - 4
        if i <= flask.charges then
            v.fill:SetHeight(h)
            v.fill:SetTexture(0.85, 0.12, 0.12, 0.95)
        elseif i == flask.charges + 1 and flask.progress > 0 then
            v.fill:SetHeight(math.max(1, h * flask.progress / 100))
            v.fill:SetTexture(0.45, 0.10, 0.10, 0.9)
        else
            v.fill:SetHeight(1)
            v.fill:SetTexture(0, 0, 0, 0)
        end
        v:Show()
    end
    flaskLabel:Show()
    dodgeBar:Show()
end

local function refreshDodge()
    if not dodgeBar:IsShown() then
        return
    end
    local left = dodgeReadyAt - GetTime()
    if left <= 0 then
        dodgeBar:SetValue(1)
        dodgeBar:SetStatusBarColor(0.85, 0.8, 0.55)
    else
        dodgeBar:SetValue(1 - left / dodgeCooldown)
        dodgeBar:SetStatusBarColor(0.45, 0.42, 0.3)
    end
end

local orbDriver = CreateFrame("Frame")
local orbSince = 0
orbDriver:SetScript("OnUpdate", function()
    orbSince = orbSince + (arg1 or 0)
    if orbSince >= 0.05 then
        orbSince = 0
        refreshOrbs()
        refreshDodge()
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
    .. "holding this many items of this quality around this item level. Form Test Pack makes the "
    .. "nearest mob lead a pack of five; Champion Pack and Rare Pack bring champions or a rare. "
    .. "All need Arpg.DevTools = 1 in the server's mangosd.conf."
ARPG_TOOLTIP_DUNGEON_TIER = "The difficulty of the next dungeon you enter first: tougher, harder-"
    .. "hitting monsters with more affixes, for more XP and better loot. Clearing a dungeon's final "
    .. "boss opens its next tier. Q drinks your flask; Space rolls."
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
    { "RowDungeonTier", "arpgDungeonTier", "Dungeon Tier", "ARPG_TOOLTIP_DUNGEON_TIER",
        0, 5, 1, nil, { "Normal", "Hard", "Brutal", "Torment I", "Torment II", "Torment III" } },
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
    -- Form Test Pack, Champion Pack, Rare Pack: the nearest mob forms a pack of five (Arpg.DevTools),
    -- through the web window's action setting (kind 9, the node field tier * 100 + size): tier 1
    -- plain, 2 with champions, 3 with a rare.
    local function packButton(key, label, tier, anchor, x, y)
        local b = CreateFrame("Button", body:GetName() .. key, prev, "BenillaOptionsRedButtonTemplate")
        b:SetWidth(140)
        b:SetPoint("TOPLEFT", anchor, "TOPLEFT", x, y)
        b:SetText(label)
        b:SetScript("OnClick", function()
            PlaySound("igMainMenuOptionCheckBoxOn")
            local n = (tonumber(GetCVar("arpgTreeAction")) or 0) + 1
            local nonce = n - math.floor(n / 100) * 100
            SetCVar("arpgTreeAction", tostring(900000 + (tier * 100 + 5) * 100 + nonce))
        end)
        return b
    end
    packButton("DevPack", "Form Test Pack", 1, drop, 150, 0)
    packButton("DevChampionPack", "Champion Pack", 2, drop, 0, -28)
    packButton("DevRarePack", "Rare Pack", 3, drop, 150, -28)

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

-- ── The ARPG passive web window ──
-- The server owns the web (Arpg/ArpgTree.h): the client hands each layout it sends to
-- ArpgTree_Update. The talent key and the talent micro button open this window in place of the
-- talent frame. Drag to pan, the wheel zooms; a click takes a node that joins the web, a right
-- click gives one back, Respec gives everything back (free, out of combat). Requests go back
-- through the session-only setting arpgTreeAction, as a number.

local WEB_REGION_RGB = {
    [0] = { 0.86, 0.36, 0.30 },  -- Crusader
    [1] = { 0.98, 0.84, 0.45 },  -- Lightbringer
    [2] = { 0.42, 0.62, 0.95 },  -- Templar
    [3] = { 0.80, 0.74, 0.60 },  -- the start and the bridges
}
local WEB_KIND_NAME = { [0] = "Start", [1] = "Small", [2] = "Notable", [3] = "Keystone" }
local WEB_SIZE = { [0] = 40, [1] = 20, [2] = 38, [3] = 50 }
local WEB_SOLID_TEX = "Interface\\Buttons\\WHITE8X8"
-- Paths are rows of small squares this far apart, in web units: solid at any zoom, and no
-- texture-coordinate tricks (a rotated route-line texture tiles outside its square).
local WEB_DOT_STEP = 8
local treeData, treeFrame, treeNonce = nil, nil, 0
local webById, webNext = {}, {}
-- The web spans about 1100 units each way, centred a little below the start.
local webView = { x = 0, y = 90, zoom = nil }

-- kind 1 take, 2 respec, 3 query, 4 give back: kind * 100000 + node * 100 + a nonce under 100.
local function treeAsk(kind, node)
    treeNonce = treeNonce + 1
    if treeNonce >= 100 then treeNonce = 1 end
    SetCVar("arpgTreeAction", tostring(kind * 100000 + (node or 0) * 100 + treeNonce))
end

local function webTaken(id)
    local n = webById[id]
    return n and n.taken == 1
end

-- Whether `n` joins a taken node.
local function webJoined(n)
    for _, m in ipairs(webNext[n.id] or {}) do
        if webTaken(m) then return true end
    end
    return false
end

-- Why `n` cannot be taken now, or nil when it can.
local function webBlocked(n)
    if n.taken == 1 then return "Taken" end
    if not webJoined(n) then return "Not joined to your web yet" end
    if treeData.spent >= treeData.total then return "No points left" end
    return nil
end

-- A path is `n` squares in `line` (a table of textures, grown as needed), on `parent`.
local function lineDots(line, parent, layer, n)
    for k = 1, n do
        if not line[k] then
            local T = parent:CreateTexture(nil, layer)
            T:SetTexture(WEB_SOLID_TEX)
            line[k] = T
        end
    end
    for k = n + 1, table.getn(line) do line[k]:Hide() end
end

-- Lay `line`'s first `n` squares evenly from (ax, ay) to (bx, by), from `parent`'s TOPLEFT, y down.
local function placeDots(line, parent, n, ax, ay, bx, by, size)
    for k = 1, n do
        local t = (k - 0.5) / n
        local T = line[k]
        T:ClearAllPoints()
        T:SetWidth(size)
        T:SetHeight(size)
        T:SetPoint("CENTER", parent, "TOPLEFT", ax + (bx - ax) * t, -(ay + (by - ay) * t))
        T:Show()
    end
end

local function colorDots(line, n, r, g, b, a)
    for k = 1, n do line[k]:SetVertexColor(r, g, b, a) end
end

local function webShowTip(button)
    local n = button.node
    if not n then return end
    local rgb = WEB_REGION_RGB[n.region] or WEB_REGION_RGB[3]
    GameTooltip:SetOwner(button, "ANCHOR_RIGHT")
    GameTooltip:SetText(n.name, rgb[1], rgb[2], rgb[3])
    GameTooltip:AddLine(WEB_KIND_NAME[n.kind] or "", 0.6, 0.6, 0.6)
    GameTooltip:AddLine(n.text, 1, 0.82, 0, 1)
    if n.kind ~= 0 then
        local blocked = webBlocked(n)
        if n.taken == 1 then
            GameTooltip:AddLine("Right-click to give it back", 0.5, 0.8, 1)
        elseif blocked then
            GameTooltip:AddLine(blocked, 1, 0.13, 0.13)
        else
            GameTooltip:AddLine("Click to take", 0, 1, 0)
        end
    end
    GameTooltip:Show()
end

local function webMakeButton(parent, id)
    local b = CreateFrame("Button", "ArpgWebNode" .. id, parent)
    b.border = b:CreateTexture(nil, "BACKGROUND")
    b.icon = b:CreateTexture(nil, "ARTWORK")
    b.icon:SetAllPoints(b)
    b.glow = b:CreateTexture(nil, "OVERLAY")
    b.glow:SetTexture("Interface\\Buttons\\ButtonHilight-Square")
    b.glow:SetBlendMode("ADD")
    b.glow:SetAllPoints(b)
    b.glow:Hide()
    b:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    b:SetScript("OnEnter", function() this.glow:Show(); webShowTip(this) end)
    b:SetScript("OnLeave", function() this.glow:Hide(); GameTooltip:Hide() end)
    b:SetScript("OnClick", function()
        local n = this.node
        if not n or n.kind == 0 then return end
        if arg1 == "RightButton" then
            if n.taken == 1 then treeAsk(4, n.id) end
        elseif not webBlocked(n) then
            treeAsk(1, n.id)
        end
    end)
    return b
end

-- Where web point (wx, wy) sits on the canvas, from its TOPLEFT, y down.
local function webToCanvas(wx, wy)
    local f = treeFrame
    return f.canvasW / 2 + (wx - webView.x) * webView.zoom, f.canvasH / 2 + (wy - webView.y) * webView.zoom
end

-- Lay every node, line and label out at the current pan and zoom.
local function webLayout()
    local f = treeFrame
    if not f or not treeData then return end
    local canvas, z = f.canvas, webView.zoom
    for _, n in ipairs(treeData.nodes) do
        local b = f.buttons[n.id]
        if b then
            local px, py = webToCanvas(n.x, n.y)
            local size = math.max(8, (WEB_SIZE[n.kind] or 16) * math.max(0.6, z * 1.6))
            b:SetWidth(size)
            b:SetHeight(size)
            b:ClearAllPoints()
            b:SetPoint("CENTER", canvas, "TOPLEFT", px, -py)
            local pad = n.kind == 1 and 1 or math.max(2, size * 0.08)
            b.border:ClearAllPoints()
            b.border:SetPoint("TOPLEFT", b, "TOPLEFT", -pad, pad)
            b.border:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", pad, -pad)
        end
    end
    local size = math.max(2, WEB_DOT_STEP * z * 1.05)
    for i, link in ipairs(treeData.links) do
        local line = f.lines[i]
        local a, b = webById[link[1]], webById[link[2]]
        if line and a and b then
            local ax, ay = webToCanvas(a.x, a.y)
            local bx, by = webToCanvas(b.x, b.y)
            placeDots(line, canvas, f.lineN[i], ax, ay, bx, by, size)
        end
    end
    for i, region in ipairs(treeData.regions) do
        local label = f.labels[i]
        if label then
            local px, py = webToCanvas(region.x, region.y)
            label:ClearAllPoints()
            label:SetPoint("CENTER", canvas, "TOPLEFT", px, -py)
        end
    end
end

-- Colour every node and line for what is taken and what can be.
local function webPaint()
    local f = treeFrame
    for _, n in ipairs(treeData.nodes) do
        local b = f.buttons[n.id]
        local rgb = WEB_REGION_RGB[n.region] or WEB_REGION_RGB[3]
        local taken = n.taken == 1
        local open = not taken and not webBlocked(n)
        if n.kind == 1 then
            b.icon:SetTexture(WEB_SOLID_TEX)
            b.icon:SetDesaturated(nil)
            b.border:SetTexture(0, 0, 0, 0.9)
            if taken then
                b.icon:SetVertexColor(rgb[1], rgb[2], rgb[3])
            elseif open then
                b.icon:SetVertexColor(0.85, 0.85, 0.85)
            else
                b.icon:SetVertexColor(0.32, 0.32, 0.34)
            end
        else
            b.icon:SetTexture(n.icon ~= "" and n.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
            b.icon:SetDesaturated(not (taken or open) and 1 or nil)
            if taken then
                b.icon:SetVertexColor(1, 1, 1)
                b.border:SetTexture(rgb[1], rgb[2], rgb[3], 1)
            elseif open then
                b.icon:SetVertexColor(0.9, 0.9, 0.9)
                b.border:SetTexture(0.75, 0.75, 0.75, 0.9)
            else
                b.icon:SetVertexColor(0.55, 0.55, 0.55)
                b.border:SetTexture(0.18, 0.18, 0.2, 0.95)
            end
            if n.kind == 3 and taken then
                b.border:SetTexture(1, 0.5, 0, 1)
            end
        end
        b.node = n
        b:Show()
    end
    for i, link in ipairs(treeData.links) do
        local line, n = f.lines[i], f.lineN[i]
        local a, b = webTaken(link[1]), webTaken(link[2])
        if a and b then
            colorDots(line, n, 1, 0.8, 0.3, 1)
        elseif a or b then
            colorDots(line, n, 0.6, 0.6, 0.62, 0.9)
        else
            colorDots(line, n, 0.28, 0.28, 0.32, 0.8)
        end
    end
    for i, region in ipairs(treeData.regions) do
        local label = f.labels[i]
        local rgb = WEB_REGION_RGB[i - 1] or WEB_REGION_RGB[3]
        label:SetText(region.name)
        label:SetTextColor(rgb[1], rgb[2], rgb[3], 0.9)
    end
    if f.tab ~= "skills" then
        f.points:SetText("Points: " .. (treeData.total - treeData.spent) .. " of " .. treeData.total .. " left")
    end
end

-- Zoom by `factor` about canvas point (cx, cy), from its TOPLEFT, y down.
local function webZoom(factor, cx, cy)
    local f = treeFrame
    local old = webView.zoom
    local new = math.max(0.25, math.min(1.6, old * factor))
    -- The web point under (cx, cy) stays under it.
    local wx = webView.x + (cx - f.canvasW / 2) / old
    local wy = webView.y + (cy - f.canvasH / 2) / old
    webView.zoom = new
    webView.x = wx - (cx - f.canvasW / 2) / new
    webView.y = wy - (cy - f.canvasH / 2) / new
    webLayout()
end

-- The cursor on the canvas, from its TOPLEFT, y down.
local function webCursor()
    local f = treeFrame
    local x, y = GetCursorPosition()
    local scale = f.canvas:GetEffectiveScale()
    return x / scale - f.view:GetLeft(), f.view:GetTop() - y / scale
end

local function treeBuild()
    if treeFrame then return treeFrame end
    local f = CreateFrame("Frame", "ArpgTreeFrame", UIParent)
    local width = math.min(980, UIParent:GetWidth() - 60)
    local height = math.min(700, UIParent:GetHeight() - 120)
    f:SetWidth(width)
    f:SetHeight(height)
    f:SetPoint("CENTER", UIParent, "CENTER", 0, 30)
    f:SetFrameStrata("DIALOG")
    f:EnableMouse(true)
    f:Hide()
    local bg = f:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(f)
    bg:SetTexture(0.03, 0.028, 0.04, 0.96)
    local edge = f:CreateTexture(nil, "BORDER")
    edge:SetPoint("TOPLEFT", f, "TOPLEFT", 0, -36)
    edge:SetPoint("TOPRIGHT", f, "TOPRIGHT", 0, -36)
    edge:SetHeight(1)
    edge:SetTexture(0.9, 0.8, 0.5, 0.35)
    f.title = f:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    f.title:SetPoint("TOP", f, "TOP", 0, -10)
    f.title:SetText("Passive Web")
    f.points = f:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    f.points:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -13)
    local close = CreateFrame("Button", "ArpgTreeFrameClose", f, "UIPanelCloseButton")
    close:SetPoint("TOPRIGHT", f, "TOPRIGHT", -2, -2)
    -- Two tabs: the passive web and the specialised skills.
    f.tabs = {}
    for i, tab in ipairs({ { "web", "Passive Web" }, { "skills", "Skills" } }) do
        local b = CreateFrame("Button", "ArpgTreeFrameTab" .. i, f, "UIPanelButtonTemplate")
        b:SetWidth(100)
        b:SetHeight(20)
        b:SetPoint("TOPRIGHT", f, "TOPRIGHT", -36 - (2 - i) * 104, -8)
        b:SetText(tab[2])
        b.tab = tab[1]
        b:SetScript("OnClick", function() ArpgTree_ShowTab(this.tab) end)
        f.tabs[i] = b
    end
    f.webPane = CreateFrame("Frame", "ArpgTreeWebPane", f)
    f.webPane:SetAllPoints(f)
    local hint = f.webPane:CreateFontString(nil, "OVERLAY", "GameFontDisableSmall")
    hint:SetPoint("BOTTOMLEFT", f, "BOTTOMLEFT", 16, 14)
    hint:SetText("Drag to move, mouse wheel to zoom. Click a node joined to your web to take it; right-click to give one back.")
    local respec = CreateFrame("Button", "ArpgTreeFrameRespec", f.webPane, "UIPanelButtonTemplate")
    respec:SetWidth(90)
    respec:SetHeight(22)
    respec:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -12, 9)
    respec:SetText("Respec")
    respec:SetScript("OnClick", function() treeAsk(2) end)

    -- The viewport clips the canvas, which holds the lines, labels and node buttons.
    local view = CreateFrame("ScrollFrame", "ArpgTreeView", f.webPane)
    view:SetPoint("TOPLEFT", f, "TOPLEFT", 4, -38)
    view:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -4, 38)
    f.canvasW, f.canvasH = width - 8, height - 76
    local canvas = CreateFrame("Frame", "ArpgTreeCanvas", view)
    canvas:SetWidth(f.canvasW)
    canvas:SetHeight(f.canvasH)
    view:SetScrollChild(canvas)
    view:EnableMouse(true)
    view:EnableMouseWheel(true)
    view:SetScript("OnMouseWheel", function()
        local cx, cy = webCursor()
        webZoom(arg1 > 0 and 1.15 or 1 / 1.15, cx, cy)
    end)
    view:SetScript("OnMouseDown", function()
        local cx, cy = webCursor()
        f.drag = { cx = cx, cy = cy, x = webView.x, y = webView.y }
    end)
    view:SetScript("OnMouseUp", function() f.drag = nil end)
    view:SetScript("OnUpdate", function()
        local d = f.drag
        if not d then return end
        local cx, cy = webCursor()
        webView.x = d.x - (cx - d.cx) / webView.zoom
        webView.y = d.y - (cy - d.cy) / webView.zoom
        webLayout()
    end)
    f.view, f.canvas = view, canvas
    f.buttons, f.lines, f.lineN, f.labels = {}, {}, {}, {}
    f:SetScript("OnHide", function() f.drag = nil end)
    if UISpecialFrames then tinsert(UISpecialFrames, "ArpgTreeFrame") end
    treeFrame = f
    return f
end

-- Take a new layout: index it, make what it needs, then paint and lay it out.
local function treeRedraw()
    if not treeData then return end
    local f = treeBuild()
    webById, webNext = {}, {}
    for _, n in ipairs(treeData.nodes) do
        webById[n.id] = n
        webNext[n.id] = {}
        if not f.buttons[n.id] then
            f.buttons[n.id] = webMakeButton(f.canvas, n.id)
        end
    end
    for i, link in ipairs(treeData.links) do
        if webNext[link[1]] and webNext[link[2]] then
            tinsert(webNext[link[1]], link[2])
            tinsert(webNext[link[2]], link[1])
        end
        local a, b = webById[link[1]], webById[link[2]]
        local length = (a and b) and math.sqrt((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)) or 0
        f.lineN[i] = math.max(1, math.ceil(length / WEB_DOT_STEP))
        f.lines[i] = f.lines[i] or {}
        lineDots(f.lines[i], f.canvas, "BACKGROUND", f.lineN[i])
    end
    for i = table.getn(treeData.links) + 1, table.getn(f.lines) do
        lineDots(f.lines[i], f.canvas, "BACKGROUND", 0)
        f.lineN[i] = 0
    end
    for i, _ in ipairs(treeData.regions) do
        if not f.labels[i] then
            f.labels[i] = f.canvas:CreateFontString(nil, "ARTWORK", "GameFontNormalLarge")
        end
    end
    for id, b in pairs(f.buttons) do
        if not webById[id] then b:Hide() end
    end
    if not webView.zoom then
        -- First open: the whole web in view.
        webView.zoom = math.min(f.canvasW, f.canvasH) / 1200
    end
    webPaint()
    webLayout()
end

-- ── The Skills tab ──
-- The server owns the skills (Arpg/ArpgSkills.h) and hands them to ArpgSkills_Update. Five slots
-- open by level; a skill in a slot takes points in its tree (click a node, right-click to give a
-- rank back). Taking a skill out of its slot gives its points back.

local SKILL_KIND_NAME = { [1] = "Modifier", [2] = "Transformer", [3] = "Synergy", [4] = "Capstone" }
local SKILL_KIND_RGB = {
    [1] = { 0.55, 0.65, 0.80 },
    [2] = { 0.902, 0.8, 0.502 },
    [3] = { 0.35, 0.80, 0.75 },
    [4] = { 1.0, 0.5, 0.0 },
}
local SKILL_COL_W, SKILL_ROW_H, SKILL_NODE = 150, 92, 40
local skillsData, skillSel, slotSel = nil, nil, 1

local function skillById(id)
    if not skillsData then return nil end
    for _, k in ipairs(skillsData.skills) do
        if k.id == id then return k end
    end
    return nil
end

-- The slot a skill sits in, or nil.
local function skillSlot(id)
    for i, slot in ipairs(skillsData.slots) do
        if slot.skill == id then return i end
    end
    return nil
end

local function skillRank(k, id)
    if id == 0 then return 1 end
    for _, n in ipairs(k.nodes) do
        if n.id == id then return n.rank end
    end
    return 0
end

-- Why a rank cannot go into node `n` of skill `k` now, or nil when it can.
local function skillBlocked(k, n)
    if not skillSlot(k.id) then return "Specialise " .. k.name .. " in a slot first" end
    if n.rank >= n.max then return "Fully learned" end
    if skillRank(k, n.parent) == 0 then return "Needs a rank in the node above" end
    if k.spent >= k.cap then return k.name .. " has all " .. k.cap .. " points it can take" end
    if skillsData.spent >= skillsData.total then return "No skill points left" end
    return nil
end

local function skillShowTip(button)
    local n, k = button.node, button.skill
    if not n or not k then return end
    GameTooltip:SetOwner(button, "ANCHOR_RIGHT")
    GameTooltip:SetText(n.name, 1, 1, 1)
    local rgb = SKILL_KIND_RGB[n.kind] or SKILL_KIND_RGB[1]
    GameTooltip:AddLine((SKILL_KIND_NAME[n.kind] or "") .. "   Rank " .. n.rank .. "/" .. n.max, rgb[1], rgb[2], rgb[3])
    GameTooltip:AddLine(n.text, 1, 0.82, 0, 1)
    local blocked = skillBlocked(k, n)
    if blocked then
        GameTooltip:AddLine(blocked, 1, 0.13, 0.13)
    else
        GameTooltip:AddLine("Click to learn", 0, 1, 0)
    end
    if n.rank > 0 then GameTooltip:AddLine("Right-click to give a rank back", 0.5, 0.8, 1) end
    GameTooltip:Show()
end

local function skillMakeNode(parent, i)
    local b = CreateFrame("Button", "ArpgSkillNode" .. i, parent)
    b.border = b:CreateTexture(nil, "BACKGROUND")
    b.border:SetPoint("TOPLEFT", b, "TOPLEFT", -3, 3)
    b.border:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", 3, -3)
    b.icon = b:CreateTexture(nil, "ARTWORK")
    b.icon:SetAllPoints(b)
    b.rank = b:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    b.rank:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", 4, -4)
    b.label = b:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    b.label:SetPoint("TOP", b, "BOTTOM", 0, -5)
    b:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    b:SetScript("OnEnter", function() skillShowTip(this) end)
    b:SetScript("OnLeave", function() GameTooltip:Hide() end)
    b:SetScript("OnClick", function()
        local n, k = this.node, this.skill
        if not n or not k or n.kind == 0 then return end
        if arg1 == "RightButton" then
            if n.rank > 0 then treeAsk(7, n.id) end
        elseif not skillBlocked(k, n) then
            treeAsk(6, n.id)
        end
    end)
    return b
end

local function skillMakeRow(parent, name, width)
    local b = CreateFrame("Button", name, parent)
    b:SetWidth(width)
    b:SetHeight(36)
    b.bg = b:CreateTexture(nil, "BACKGROUND")
    b.bg:SetAllPoints(b)
    b.icon = b:CreateTexture(nil, "ARTWORK")
    b.icon:SetWidth(30)
    b.icon:SetHeight(30)
    b.icon:SetPoint("LEFT", b, "LEFT", 3, 0)
    b.text = b:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    b.text:SetPoint("TOPLEFT", b, "TOPLEFT", 40, -4)
    b.text:SetJustifyH("LEFT")
    b.sub = b:CreateFontString(nil, "OVERLAY", "GameFontDisableSmall")
    b.sub:SetPoint("BOTTOMLEFT", b, "BOTTOMLEFT", 40, 5)
    b.sub:SetJustifyH("LEFT")
    b.glow = b:CreateTexture(nil, "HIGHLIGHT")
    b.glow:SetAllPoints(b)
    b.glow:SetTexture(1, 1, 1, 0.08)
    return b
end

local function skillsBuild(f)
    if f.skillPane then return f.skillPane end
    local p = CreateFrame("Frame", "ArpgSkillPane", f)
    p:SetPoint("TOPLEFT", f, "TOPLEFT", 4, -38)
    p:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -4, 4)
    p:Hide()
    local slotsLabel = p:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    slotsLabel:SetPoint("TOPLEFT", p, "TOPLEFT", 12, -8)
    slotsLabel:SetText("Slots")
    p.slots = {}
    for i = 1, 5 do
        local row = skillMakeRow(p, "ArpgSkillSlot" .. i, 240)
        row:SetPoint("TOPLEFT", p, "TOPLEFT", 10, -26 - (i - 1) * 40)
        row.index = i
        row:SetScript("OnClick", function()
            slotSel = this.index
            local slot = skillsData and skillsData.slots[this.index]
            if slot and slot.skill ~= 0 then skillSel = slot.skill end
            ArpgSkills_Redraw()
        end)
        p.slots[i] = row
    end
    local skillsLabel = p:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    skillsLabel:SetPoint("TOPLEFT", p, "TOPLEFT", 12, -232)
    skillsLabel:SetText("Skills")
    p.rows = {}
    for i = 1, 8 do
        local row = skillMakeRow(p, "ArpgSkillRow" .. i, 240)
        row:SetPoint("TOPLEFT", p, "TOPLEFT", 10, -250 - (i - 1) * 40)
        row:SetScript("OnClick", function()
            skillSel = this.skillId
            ArpgSkills_Redraw()
        end)
        p.rows[i] = row
    end
    local function button(name, label, x)
        local b = CreateFrame("Button", name, p, "UIPanelButtonTemplate")
        b:SetWidth(110)
        b:SetHeight(22)
        b:SetPoint("BOTTOMRIGHT", p, "BOTTOMRIGHT", x, 6)
        b:SetText(label)
        return b
    end
    -- Put the shown skill in the chosen slot (or the first open, empty one).
    p.put = button("ArpgSkillPut", "Specialise", -244)
    p.put:SetScript("OnClick", function()
        if not skillsData or not skillSel then return end
        local slot = skillsData.slots[slotSel]
        local target = nil
        if slot and slot.level <= skillsData.level then target = slotSel end
        if not target or (skillsData.slots[target].skill ~= 0 and skillsData.slots[target].skill ~= skillSel) then
            for i, s in ipairs(skillsData.slots) do
                if s.skill == 0 and s.level <= skillsData.level then target = i break end
            end
        end
        if target then treeAsk(5, (target - 1) * 100 + skillSel) end
    end)
    p.take = button("ArpgSkillTake", "Take Out", -128)
    p.take:SetScript("OnClick", function()
        local slot = skillSel and skillSlot(skillSel)
        if slot then treeAsk(5, (slot - 1) * 100) end
    end)
    p.respec = button("ArpgSkillRespec", "Respec Skill", -12)
    p.respec:SetScript("OnClick", function()
        if skillSel then treeAsk(8, skillSel) end
    end)
    -- The tree, right of the lists.
    p.tree = CreateFrame("Frame", "ArpgSkillTree", p)
    p.tree:SetPoint("TOPLEFT", p, "TOPLEFT", 270, -8)
    p.tree:SetPoint("BOTTOMRIGHT", p, "BOTTOMRIGHT", -8, 36)
    local shade = p.tree:CreateTexture(nil, "BACKGROUND")
    shade:SetAllPoints(p.tree)
    shade:SetTexture(1, 1, 1, 0.03)
    p.treeTitle = p.tree:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    p.treeTitle:SetPoint("TOPLEFT", p.tree, "TOPLEFT", 12, -10)
    p.treeText = p.tree:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    p.treeText:SetPoint("TOPLEFT", p.tree, "TOPLEFT", 12, -32)
    p.treeText:SetJustifyH("LEFT")
    p.nodes, p.lines, p.branches = {}, {}, {}
    f.skillPane = p
    return p
end

local function skillsPaint()
    local f = treeFrame
    if not f or not skillsData then return end
    local p = skillsBuild(f)
    if f.tab == "skills" then
        f.points:SetText("Skill points: " .. (skillsData.total - skillsData.spent) .. " of " .. skillsData.total .. " left")
    end
    if not skillSel or not skillById(skillSel) then
        local first = skillsData.skills[1]
        skillSel = first and first.id
    end
    for i, row in ipairs(p.slots) do
        local slot = skillsData.slots[i]
        if slot then
            local k = skillById(slot.skill)
            local open = slot.level <= skillsData.level
            if k then
                row.icon:SetTexture(k.icon ~= "" and k.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
                row.text:SetText(k.name)
                row.sub:SetText("Slot " .. i .. "   " .. k.spent .. "/" .. k.cap .. " points")
            else
                row.icon:SetTexture("Interface\\Buttons\\UI-EmptySlot")
                row.text:SetText(open and "Empty" or "Locked")
                row.sub:SetText(open and ("Slot " .. i) or ("Opens at level " .. slot.level))
            end
            row.icon:SetDesaturated(not open and 1 or nil)
            row.bg:SetTexture(1, 0.82, 0, i == slotSel and 0.18 or 0.05)
            row:Show()
        else
            row:Hide()
        end
    end
    for i, row in ipairs(p.rows) do
        local k = skillsData.skills[i]
        if k then
            row.skillId = k.id
            row.icon:SetTexture(k.icon ~= "" and k.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
            row.text:SetText(k.name)
            local slot = skillSlot(k.id)
            row.sub:SetText(slot and ("In slot " .. slot) or "Not specialised")
            row.bg:SetTexture(1, 1, 1, k.id == skillSel and 0.16 or 0.04)
            row:Show()
        else
            row:Hide()
        end
    end

    -- The shown skill's tree: the root on top, a column per branch, a row per depth.
    local k = skillById(skillSel)
    for _, b in pairs(p.nodes) do b:Hide() end
    for _, line in pairs(p.lines) do lineDots(line, p.tree, "BORDER", 0) end
    for _, l in pairs(p.branches) do l:Hide() end
    if not k then return end
    p.treeTitle:SetText(k.name .. "   " .. k.spent .. "/" .. k.cap)
    p.treeText:SetText(k.text)
    -- The tree's size from the window's own (set, not anchored), as its layout may not have run yet.
    local width, height = f:GetWidth() - 286, f:GetHeight() - 86
    local left = (width - 2 * SKILL_COL_W) / 2
    local function at(col, row)
        return left + col * SKILL_COL_W, 92 + row * SKILL_ROW_H
    end
    local all = { { id = 0, kind = 0, col = 1, row = 0, parent = -1, max = 0, rank = 1, icon = k.icon, name = k.name, text = k.text } }
    for _, n in ipairs(k.nodes) do tinsert(all, n) end
    local pos = {}
    for i, n in ipairs(all) do
        local b = p.nodes[i]
        if not b then
            b = skillMakeNode(p.tree, i)
            p.nodes[i] = b
        end
        local x, y = at(n.col, n.row)
        pos[n.id] = { x, y }
        local size = n.kind == 4 and SKILL_NODE + 8 or (n.kind == 0 and SKILL_NODE + 12 or SKILL_NODE)
        b:SetWidth(size)
        b:SetHeight(size)
        b:ClearAllPoints()
        b:SetPoint("CENTER", p.tree, "TOPLEFT", x, -y)
        b.node, b.skill = n, k
        b.icon:SetTexture(n.icon ~= "" and n.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
        b.label:SetText(n.kind == 0 and "" or n.name)
        local rgb = SKILL_KIND_RGB[n.kind] or { 1, 0.82, 0 }
        local blocked = n.kind ~= 0 and skillBlocked(k, n)
        local lit = n.kind == 0 or n.rank > 0 or not blocked
        b.icon:SetDesaturated(not lit and 1 or nil)
        if n.kind == 0 then
            b.border:SetTexture(1, 0.82, 0, 1)
            b.rank:SetText("")
        else
            b.border:SetTexture(rgb[1], rgb[2], rgb[3], lit and 0.95 or 0.35)
            b.rank:SetText(n.rank .. "/" .. n.max)
            if n.rank >= n.max then
                b.rank:SetTextColor(1, 0.82, 0)
            elseif n.rank > 0 then
                b.rank:SetTextColor(0, 1, 0)
            else
                b.rank:SetTextColor(0.8, 0.8, 0.8)
            end
        end
        b:Show()
    end
    local li = 0
    for _, n in ipairs(k.nodes) do
        local from, to = pos[n.parent], pos[n.id]
        if from and to then
            li = li + 1
            p.lines[li] = p.lines[li] or {}
            local line = p.lines[li]
            local dx, dy = to[1] - from[1], to[2] - from[2]
            local count = math.max(1, math.ceil(math.sqrt(dx * dx + dy * dy) / 4))
            lineDots(line, p.tree, "BORDER", count)
            placeDots(line, p.tree, count, from[1], from[2], to[1], to[2], 4)
            if n.rank > 0 then
                colorDots(line, count, 1, 0.8, 0.3, 1)
            else
                colorDots(line, count, 0.35, 0.35, 0.4, 0.9)
            end
        end
    end
    for i, name in ipairs(k.branches) do
        local l = p.branches[i]
        if not l then
            l = p.tree:CreateFontString(nil, "OVERLAY", "GameFontNormal")
            p.branches[i] = l
        end
        local x, y = at(i - 1, 1)
        l:ClearAllPoints()
        l:SetPoint("BOTTOM", p.tree, "TOPLEFT", x, -(y - SKILL_NODE / 2 - 14))
        l:SetText(name)
        l:Show()
    end
end

function ArpgSkills_Redraw()
    skillsPaint()
end

function ArpgSkills_Update(skills)
    skillsData = skills
    if treeFrame and treeFrame:IsVisible() and treeFrame.tab == "skills" then skillsPaint() end
end

function ArpgTree_ShowTab(tab)
    local f = treeBuild()
    f.tab = tab
    for _, b in ipairs(f.tabs) do
        if b.tab == tab then b:Disable() else b:Enable() end
    end
    if tab == "skills" then
        f.title:SetText("Skills")
        f.webPane:Hide()
        skillsBuild(f):Show()
        skillsPaint()
    else
        f.title:SetText("Passive Web")
        if f.skillPane then f.skillPane:Hide() end
        f.webPane:Show()
        treeRedraw()
    end
end

function ArpgTree_Update(tree)
    treeData = tree
    if treeFrame and treeFrame:IsVisible() and treeFrame.tab ~= "skills" then treeRedraw() end
end

function ArpgTree_Toggle()
    if not treeData and not skillsData then
        treeAsk(3)
        DEFAULT_CHAT_FRAME:AddMessage("The passive web has not arrived from the server yet.")
        return
    end
    local f = treeBuild()
    if f:IsVisible() then
        f:Hide()
    else
        f:Show()
        ArpgTree_ShowTab(f.tab or (treeData and "web" or "skills"))
    end
end

-- Spells without ranks: the spellbook shows each spell without its "Rank N".
if SpellButton_UpdateButton then
    local stockUpdateButton = SpellButton_UpdateButton
    SpellButton_UpdateButton = function()
        stockUpdateButton()
        local sub = this and getglobal(this:GetName() .. "SubSpellName")
        local text = sub and sub:GetText()
        if text and string.find(text, "^Rank %d+$") then sub:SetText("") end
    end
end

-- The talent key and the talent micro button open the tree instead.
ToggleTalentFrame = ArpgTree_Toggle

-- A level gives a point: ask for the fresh count.
local treeEvents = CreateFrame("Frame")
treeEvents:RegisterEvent("PLAYER_LEVEL_UP")
treeEvents:SetScript("OnEvent", function() treeAsk(3) end)
