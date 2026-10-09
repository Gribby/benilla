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
local WEB_LINE_TEX = "Interface\\TaxiFrame\\UI-Taxi-Line"
local WEB_DOT_TEX = "Interface\\TaxiFrame\\UI-Taxi-Icon-White"
local WEB_LINE_FACTOR = 32 / 30
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

-- A texture drawn as a line from (sx, sy) to (ex, ey), canvas BOTTOMLEFT coordinates, `w` wide:
-- the taxi map's route line, its texture rotated by texture coordinates.
local function webLine(T, C, sx, sy, ex, ey, w)
    local dx, dy = ex - sx, ey - sy
    local cx, cy = (sx + ex) / 2, (sy + ey) / 2
    if dx < 0 then dx, dy = -dx, -dy end
    local l = math.sqrt(dx * dx + dy * dy)
    T:ClearAllPoints()
    if l == 0 then
        T:SetTexCoord(0, 0, 0, 0, 0, 0, 0, 0)
        T:SetPoint("BOTTOMLEFT", C, "BOTTOMLEFT", cx, cy)
        T:SetPoint("TOPRIGHT", C, "BOTTOMLEFT", cx, cy)
        return
    end
    local s, c = -dy / l, dx / l
    local sc = s * c
    local Bwid, Bhgt, BLx, BLy, TLx, TLy, TRx, TRy, BRx, BRy
    if dy >= 0 then
        Bwid = ((l * c) - (w * s)) * WEB_LINE_FACTOR / 2
        Bhgt = ((w * c) - (l * s)) * WEB_LINE_FACTOR / 2
        BLx, BLy, BRy = (w / l) * sc, s * s, (l / w) * sc
        BRx, TLx, TLy, TRx = 1 - BLy, BLy, 1 - BRy, 1 - BLx
        TRy = BRx
    else
        Bwid = ((l * c) + (w * s)) * WEB_LINE_FACTOR / 2
        Bhgt = ((w * c) + (l * s)) * WEB_LINE_FACTOR / 2
        BLx, BLy, BRx = s * s, -(l / w) * sc, 1 + (w / l) * sc
        BRy, TLx, TLy, TRy = BLx, 1 - BRx, 1 - BLx, 1 - BLy
        TRx = TLy
    end
    T:SetTexCoord(TLx, TLy, BLx, BLy, TRx, TRy, BRx, BRy)
    T:SetPoint("BOTTOMLEFT", C, "BOTTOMLEFT", cx - Bwid, cy - Bhgt)
    T:SetPoint("TOPRIGHT", C, "BOTTOMLEFT", cx + Bwid, cy + Bhgt)
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
            local pad = n.kind == 1 and 0 or math.max(2, size * 0.08)
            b.border:ClearAllPoints()
            b.border:SetPoint("TOPLEFT", b, "TOPLEFT", -pad, pad)
            b.border:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", pad, -pad)
        end
    end
    local w = math.max(3, 9 * z)
    for i, link in ipairs(treeData.links) do
        local T = f.lines[i]
        local a, b = webById[link[1]], webById[link[2]]
        if T and a and b then
            local ax, ay = webToCanvas(a.x, a.y)
            local bx, by = webToCanvas(b.x, b.y)
            webLine(T, canvas, ax, f.canvasH - ay, bx, f.canvasH - by, w)
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
            b.icon:SetTexture(WEB_DOT_TEX)
            b.border:SetTexture(0, 0, 0, 0)
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
        local T = f.lines[i]
        local a, b = webTaken(link[1]), webTaken(link[2])
        if a and b then
            T:SetVertexColor(1, 0.8, 0.3, 1)
        elseif a or b then
            T:SetVertexColor(0.6, 0.6, 0.62, 0.9)
        else
            T:SetVertexColor(0.28, 0.28, 0.32, 0.8)
        end
    end
    for i, region in ipairs(treeData.regions) do
        local label = f.labels[i]
        local rgb = WEB_REGION_RGB[i - 1] or WEB_REGION_RGB[3]
        label:SetText(region.name)
        label:SetTextColor(rgb[1], rgb[2], rgb[3], 0.9)
    end
    f.points:SetText("Points: " .. (treeData.total - treeData.spent) .. " of " .. treeData.total .. " left")
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
    local title = f:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    title:SetPoint("TOP", f, "TOP", 0, -10)
    title:SetText("Passive Web")
    f.points = f:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    f.points:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -13)
    local hint = f:CreateFontString(nil, "OVERLAY", "GameFontDisableSmall")
    hint:SetPoint("BOTTOMLEFT", f, "BOTTOMLEFT", 16, 14)
    hint:SetText("Drag to move, mouse wheel to zoom. Click a node joined to your web to take it; right-click to give one back.")
    local close = CreateFrame("Button", "ArpgTreeFrameClose", f, "UIPanelCloseButton")
    close:SetPoint("TOPRIGHT", f, "TOPRIGHT", -2, -2)
    local respec = CreateFrame("Button", "ArpgTreeFrameRespec", f, "UIPanelButtonTemplate")
    respec:SetWidth(90)
    respec:SetHeight(22)
    respec:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -12, 9)
    respec:SetText("Respec")
    respec:SetScript("OnClick", function() treeAsk(2) end)

    -- The viewport clips the canvas, which holds the lines, labels and node buttons.
    local view = CreateFrame("ScrollFrame", "ArpgTreeView", f)
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
    f.buttons, f.lines, f.labels = {}, {}, {}
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
        if not f.lines[i] then
            local T = f.canvas:CreateTexture(nil, "BACKGROUND")
            T:SetTexture(WEB_LINE_TEX)
            f.lines[i] = T
        end
        f.lines[i]:Show()
    end
    for i = table.getn(treeData.links) + 1, table.getn(f.lines) do f.lines[i]:Hide() end
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

function ArpgTree_Update(tree)
    treeData = tree
    if treeFrame and treeFrame:IsVisible() then treeRedraw() end
end

function ArpgTree_Toggle()
    if not treeData then
        treeAsk(3)
        DEFAULT_CHAT_FRAME:AddMessage("The passive web has not arrived from the server yet.")
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
