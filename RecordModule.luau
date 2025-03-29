local record_module = {}

function record_module.add(records)
	for obby_name, types in pairs(records) do
		for mode, entry in pairs(types) do
			local player_name = entry[1]
			local new_time = tonumber(entry[2])
			if not new_time then
				continue
			end

			local leaderboard_name = obby_name .. mode .. "Leaderboard"
			local leaderboard = workspace:FindFirstChild(leaderboard_name)

			if leaderboard then
				local name_holder = leaderboard:FindFirstChild("HolderName")
				local time_holder = leaderboard:FindFirstChild("RecordTime")

				if
					name_holder
					and time_holder
					and name_holder:FindFirstChild("SurfaceGui")
					and time_holder:FindFirstChild("SurfaceGui")
				then
					local name_label = name_holder.SurfaceGui:FindFirstChild("Label")
					local time_label = time_holder.SurfaceGui:FindFirstChild("Label")

					local current_time = tonumber(time_label and time_label.Text or "")
					if not current_time or new_time < current_time then
						if name_label then
							name_label.Text = player_name
						end
						if time_label then
							time_label.Text = string.format("%.3f", new_time)
						end
					end
				end
			end
		end
	end
end

function record_module.get_records()
	local records = {}

	for _, leaderboard in ipairs(workspace:GetChildren()) do
		if leaderboard:IsA("Folder") or leaderboard:IsA("Model") then
			local name = leaderboard.Name
			local mode = nil
			local obby_name = nil

			if name:find("BouncelessLeaderboard") then
				mode = "Bounceless"
				obby_name = name:gsub("BouncelessLeaderboard", "")
			elseif name:find("BounceLeaderboard") then
				mode = "Bounce"
				obby_name = name:gsub("BounceLeaderboard", "")
			end

			if mode and obby_name then
				local name_holder = leaderboard:FindFirstChild("HolderName")
				local time_holder = leaderboard:FindFirstChild("RecordTime")

				if
					name_holder
					and time_holder
					and name_holder:FindFirstChild("SurfaceGui")
					and time_holder:FindFirstChild("SurfaceGui")
				then
					local name_label = name_holder.SurfaceGui:FindFirstChild("Label")
					local time_label = time_holder.SurfaceGui:FindFirstChild("Label")

					local player = name_label and name_label.Text or ""
					local time_text = time_label and time_label.Text or ""
					local time = tonumber(time_text)

					if player:lower() ~= "n/a" and time_text:lower() ~= "n/a" and player ~= "" and time then
						records[obby_name] = records[obby_name] or {}
						records[obby_name][mode] = { player, time }
					end
				end
			end
		end
	end

	-- convert to formatted string
	local output = "{\n"
	for obby_name, types in pairs(records) do
		output ..= string.format('  ["%s"] = {\n', obby_name)
		for mode, data in pairs(types) do
			output ..= string.format('    ["%s"] = { "%s", %.3f },\n', mode, data[1], data[2])
		end
		output ..= "  },\n"
	end
	output ..= "}"
	return output
end

return record_module
