local SUBSTRING = { 1, 2, 3 }

function generateTestCase()
  local inputs = {}
  for i = 1, 15 do
    inputs[i] = math.random(0, 9)
  end

  -- Randomly put 0 to 3 substrings in the sequence
  for _ = 1, math.random(0, 3) do
    local start = math.random(0, 15 - #SUBSTRING)
    for i = 1, #SUBSTRING do
      inputs[start + i] = SUBSTRING[i]
    end
  end

  -- Hacky: use regex to replace all substring
  local outputs = {}
  for c in table.concat(inputs):gsub(table.concat(SUBSTRING), ""):gmatch(".") do
    outputs[#outputs + 1] = string.byte(c) - string.byte("0")
  end

  return inputs, outputs
end
