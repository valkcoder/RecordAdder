local record_module = {}

function record_module.add(data)
	local ctt2_mode = data.CTT2Mode == true

	for obby_name, modes in pairs(data) do
		if obby_name == "CTT2Mode" then
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
						time_label.Text = tostring(time)
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
				for _, obby in ipairs(lbs_folder:GetChildren()) do
					if obby:IsA("Folder") then
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
		output ..= '  ["' .. obby_name .. '"] = {\n'
		for mode, record in pairs(modes) do
			output ..= string.format('    ["%s"] = { "%s", %.3f },\n', mode, record[1], record[2])
		end
		output ..= "  },\n"
	end
	output ..= "}"
	return output
end

return record_module
