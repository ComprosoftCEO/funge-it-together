local NOT = 0
local AND = 1
local OR = 2
local XOR = 3
local NOR = -1
local NAND = -2
local XNOR = -3

function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 5 do -- 3 * 5 = 15
    local op = math.random(-3, 3)
    local a = math.random(0, 1)
    local b = math.random(0, 1)

    inputs[#inputs + 1] = op
    if op == NOT then
      -- Not only requires one input
      inputs[#inputs + 1] = a
    else
      -- Other operands require two inputs
      inputs[#inputs + 1] = a
      inputs[#inputs + 1] = b
    end

    if op == NOT then
      outputs[#outputs + 1] = 1 - a
    elseif op == AND then
      outputs[#outputs + 1] = a * b
    elseif op == OR then
      outputs[#outputs + 1] = math.min(1, a + b)
    elseif op == XOR then
      outputs[#outputs + 1] = boolToNumber(a ~= b)
    elseif op == NAND then
      outputs[#outputs + 1] = 1 - (a * b)
    elseif op == NOR then
      outputs[#outputs + 1] = 1 - math.min(1, a + b)
    elseif op == XNOR then
      outputs[#outputs + 1] = 1 - boolToNumber(a ~= b)
    end
  end

  return inputs, outputs
end

function boolToNumber(b)
  if b then
    return 1
  else
    return 0
  end
end
