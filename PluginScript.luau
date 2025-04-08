local HttpService = game:GetService("HttpService")
local Toolbar = plugin:CreateToolbar("Record Adder")
local button = Toolbar:CreateButton("Toggle Live Sync", "Toggle record sync", "")
button.ClickableWhenViewportHidden = true

local enabled = false

local function fetchRecords()
	local success, response = pcall(function()
		return HttpService:GetAsync("http://localhost:14855")
	end)

	if success then
		local ok, data = pcall(function()
			return HttpService:JSONDecode(response)
		end)

		if ok and typeof(data) == "table" then
			local record_module = require(game.ReplicatedStorage:WaitForChild("Modules"):WaitForChild("RecordModule"))
			record_module.add(data)
		else
			warn("Failed to parse JSON from response")
		end
	else
		warn("Failed to fetch from record adder:", response)
	end
end

task.spawn(function()
	while true do
		if enabled then
			fetchRecords()
		end
		task.wait(2)
	end
end)

button.Click:Connect(function()
	enabled = not enabled
	button:SetActive(enabled)
	print("Record sync " .. (enabled and "enabled" or "disabled"))
end)
