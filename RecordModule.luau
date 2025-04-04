local record_module = {}

local MAIN_OBBY_CATEGORIES = {
	Bounce = "B",
	Bounceless = "NB",
	NoPlat = "NT",
}

function record_module.add(data)
	local ctt2_mode = data.CTT2Mode == true

	for obby_name, modes in pairs(data) do
		if obby_name == "CTT2Mode" then
			continue
		end

		if obby_name == "MainObby" and ctt2_mode then
			local base = game.Workspace:FindFirstChild("MISC")
			if base then
				base = base:FindFirstChild("LBS")
			end
			if base then
				base = base:FindFirstChild("MO")
			end
			if not base then
				continue
			end

			local category_map = {
				Bounce = "B",
				Bounceless = "NB",
				NoPlat = "NT",
			}

			for category, records in pairs(modes) do
				local tag = category_map[category]
				if not tag then
					continue
				end

				local folder = base:FindFirstChild(tag)
				if not folder then
					continue
				end

				local sf = folder:FindFirstChild("LB")
				sf = sf and sf:FindFirstChild("Leaderboard")
				sf = sf and sf:FindFirstChild("ScrollingFrame")
				if not sf then
					continue
				end

				for i = 1, #records do
					local entry = sf:FindFirstChild(tostring(i))
					local data = records[i]
					if entry and data then
						local plr_field = i == 1 and "plr" or "plr2"
						local time_field = i == 1 and "time" or "time2"

						local plr_label = entry:FindFirstChild(plr_field)
						local time_label = entry:FindFirstChild(time_field)

						if plr_label and plr_label:IsA("TextLabel") then
							plr_label.Text = data[1]
						end

						if time_label and time_label:IsA("TextLabel") then
							time_label.Text = string.format("%.3f", data[2])
						end
					end
				end
			end

			continue
		end

		for mode, values in pairs(modes) do
			local player = values[1]
			local time = values[2]
			local bounce = mode == "Bounce"

			local leaderboard = nil

			if ctt2_mode then
				local folder = game.Workspace:FindFirstChild("MISC")
				if folder then
					folder = folder:FindFirstChild("LBS")
					if folder then
						local obby_folder = folder:FindFirstChild(obby_name:upper())
						if obby_folder then
							local subfolder = obby_folder:FindFirstChild(bounce and "B" or "NB")
							if subfolder then
								leaderboard = subfolder:FindFirstChild("Leaderboard")
							end
						end
					end
				end
			else
				local suffix = bounce and "BounceLeaderboard" or "BouncelessLeaderboard"
				leaderboard = game.Workspace:FindFirstChild(obby_name .. suffix)
			end

			if leaderboard then
				local name_label = leaderboard:FindFirstChild("plr")
				local time_label = leaderboard:FindFirstChild("time")
				if name_label and name_label:IsA("TextLabel") and time_label and time_label:IsA("TextLabel") then
					local current_time = tonumber(time_label.Text)
					if not current_time or time < current_time then
						name_label.Text = player
						time_label.Text = string.format("%.3f", time)
					end
				end
			end
		end
	end
end

function record_module.get_records(ctt2_mode)
	local records = {}

	local function insert_record(obby_name, bounce, player, time)
		if not records[obby_name] then
			records[obby_name] = {}
		end
		records[obby_name][bounce and "Bounce" or "Bounceless"] = { player, time }
	end

	local function is_valid_label(label)
		return label and label:IsA("TextLabel") and label.Text and label.Text ~= "N/A" and label.Text ~= ""
	end

	if ctt2_mode then
		local lbs_folder = game.Workspace:FindFirstChild("MISC")
		if lbs_folder then
			lbs_folder = lbs_folder:FindFirstChild("LBS")
			if lbs_folder then
				-- Normal Obbies
				for _, obby in ipairs(lbs_folder:GetChildren()) do
					if obby:IsA("Folder") and obby.Name ~= "MO" then
						for _, bounce_data in ipairs({ { true, "B" }, { false, "NB" } }) do
							local bounce = bounce_data[1]
							local tag = bounce_data[2]
							local path = obby:FindFirstChild(tag)
							local leaderboard = path and path:FindFirstChild("Leaderboard")
							if leaderboard then
								local plr = leaderboard:FindFirstChild("plr")
								local time = leaderboard:FindFirstChild("time")
								if is_valid_label(plr) and is_valid_label(time) then
									local player = plr.Text
									local t = tonumber(time.Text)
									if t then
										insert_record(obby.Name, bounce, player, t)
									end
								end
							end
						end
					end
				end

				-- Main Obby
				local mo_folder = lbs_folder:FindFirstChild("MO")
				if mo_folder then
					local main_obby_data = {}

					for cat, tag in pairs(MAIN_OBBY_CATEGORIES) do
						local folder = mo_folder:FindFirstChild(tag)
						local scrolling = folder and folder:FindFirstChild("LB")
						scrolling = scrolling and scrolling:FindFirstChild("Leaderboard")
						scrolling = scrolling and scrolling:FindFirstChild("ScrollingFrame")

						local results = {}

						if scrolling then
							for i = 1, 12 do
								local entry = scrolling:FindFirstChild(tostring(i))
								if entry then
									local plr_label = entry:FindFirstChild(i == 1 and "plr" or "plr2")
									local time_label = entry:FindFirstChild(i == 1 and "time" or "time2")

									local player = plr_label and plr_label:IsA("TextLabel") and plr_label.Text or nil
									local t = time_label and time_label:IsA("TextLabel") and tonumber(time_label.Text)
										or nil

									if player and player ~= "" and player ~= "N/A" and t then
										table.insert(results, { player, t })
									end
								end
							end
						end

						if #results > 0 then
							main_obby_data[cat] = results
						end
					end

					if next(main_obby_data) then
						records.MainObby = main_obby_data
					end
				end
			end
		end
	else
		for _, obj in ipairs(game.Workspace:GetChildren()) do
			if obj:IsA("Folder") or obj:IsA("Model") then
				local name = obj.Name
				local bounce, obby_name = nil, nil

				if name:find("BounceLeaderboard") then
					bounce = true
					obby_name = name:gsub("BounceLeaderboard", "")
				elseif name:find("BouncelessLeaderboard") then
					bounce = false
					obby_name = name:gsub("BouncelessLeaderboard", "")
				end

				if bounce ~= nil and obby_name then
					local leaderboard = obj:FindFirstChild("Leaderboard")
					if leaderboard then
						local plr = leaderboard:FindFirstChild("plr")
						local time = leaderboard:FindFirstChild("time")
						if is_valid_label(plr) and is_valid_label(time) then
							local player = plr.Text
							local t = tonumber(time.Text)
							if t then
								insert_record(obby_name, bounce, player, t)
							end
						end
					end
				end
			end
		end
	end

	local output = '{\n  ["CTT2Mode"] = ' .. tostring(ctt2_mode) .. ",\n"
	for obby_name, modes in pairs(records) do
		if obby_name == "MainObby" then
			output ..= '  ["MainObby"] = {\n'
			for cat, entries in pairs(modes) do
				output ..= '    ["' .. cat .. '"] = {\n'
				for _, entry in ipairs(entries) do
					output ..= string.format('      { "%s", %.3f },\n', entry[1], entry[2])
				end
				output ..= "    },\n"
			end
			output ..= "  },\n"
		else
			output ..= '  ["' .. obby_name .. '"] = {\n'
			for mode, record in pairs(modes) do
				output ..= string.format('    ["%s"] = { "%s", %.3f },\n', mode, record[1], record[2])
			end
			output ..= "  },\n"
		end
	end
	output ..= "}"
	return output
end

return record_module
