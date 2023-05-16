-- Define our functions
local PUSH = 1
local POP = 2
local SWAP = 3
local ADD = 4
local SUB = 5
local OUTPUT = 6

-- 4 to 5 values
-- Leaves 2 numbers on the stack
local STEP_1_TEMPLATES = {
  { PUSH, 0, PUSH, 0 },
  { PUSH, 0, PUSH, 0, SWAP },
}

-- 3 values
-- Leaves at least 2 numbers on the stack
local STEP_2_TEMPLATES = {
  { ADD,    PUSH, 0 },
  { PUSH,   0,    ADD },
  { SUB,    PUSH, 0 },
  { PUSH,   0,    SUB },
  { PUSH,   0,    POP },
  { POP,    PUSH, 0 },
  { PUSH,   0,    SWAP },
  { SWAP,   PUSH, 0 },
  { OUTPUT, PUSH, 0 },
  { PUSH,   0,    OUTPUT },
}

-- 3-5 values
-- Always has at least one output
local STEP_3_TEMPATES = {
  { POP,    PUSH,   0,    OUTPUT },
  { PUSH,   0,      SWAP, POP,   OUTPUT },
  { ADD,    OUTPUT, PUSH, 0,     OUTPUT },
  { POP,    PUSH,   0,    ADD,   OUTPUT },
  { POP,    PUSH,   0,    SUB,   OUTPUT },
  { OUTPUT, PUSH,   0,    ADD,   OUTPUT },
  { OUTPUT, PUSH,   0,    SUB,   OUTPUT },
}


function generateTestCase()
  local step_1 = STEP_1_TEMPLATES[math.random(1, #STEP_1_TEMPLATES)]
  local step_2 = STEP_2_TEMPLATES[math.random(1, #STEP_2_TEMPLATES)]
  local step_3 = STEP_3_TEMPATES[math.random(1, #STEP_3_TEMPATES)]

  local merged_steps = combineLists(step_1, step_2, step_3)
  for i = 1, #merged_steps do
    if merged_steps[i] == 0 then
      merged_steps[i] = math.random(-99, 99)
    end
  end

  return merged_steps, execute(merged_steps)
end

function combineLists(...)
  local Output = {}
  local i = 1
  for x, list in ipairs({ ... }) do
    for y, item in ipairs(list) do
      table.insert(Output, i, item)
      i = i + 1
    end
  end
  return Output
end

-- Simulate the virtual machine execution
function execute(steps)
  local stack = {}

  function stack:push(x)
    self[#self + 1] = x
  end

  function stack:pop()
    local val = self[#self]
    self[#self] = nil
    return val
  end

  function stack:swap()
    local v2 = self:pop()
    local v1 = self:pop()
    self:push(v2)
    self:push(v1)
  end

  function stack:add()
    local v2 = self:pop()
    local v1 = self:pop()
    self:push(v1 + v2)
  end

  function stack:sub()
    local v2 = self:pop()
    local v1 = self:pop()
    self:push(v1 - v2)
  end

  local output = {}
  local i = 1
  while i <= #steps do
    if steps[i] == PUSH then
      i = i + 1
      stack:push(steps[i])
    elseif steps[i] == POP then
      stack:pop()
    elseif steps[i] == SWAP then
      stack:swap()
    elseif steps[i] == ADD then
      stack:add()
    elseif steps[i] == SUB then
      stack:sub()
    elseif steps[i] == OUTPUT then
      output[#output + 1] = stack:pop()
    end

    i = i + 1 -- Next instruction
  end

  return output
end
