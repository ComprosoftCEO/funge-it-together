local READ = 1
local WRITE = -1

local NUM_ADDR = 5 -- 0 to 4

local ADDR_1 = 2
local ADDR_2 = 3
local RAND_ADDR = 4

local TEMPLATES = {
  { WRITE, ADDR_1,    0,     WRITE,  ADDR_2,    0,     WRITE,     ADDR_1,    0,    READ,      ADDR_2, READ,     ADDR_1 },
  { READ,  RAND_ADDR, WRITE, ADDR_1, 0,         WRITE, ADDR_2,    0,         READ, ADDR_1,    READ,   ADDR_2 },
  { WRITE, RAND_ADDR, 0,     WRITE,  RAND_ADDR, 0,     READ,      RAND_ADDR, READ, RAND_ADDR, READ,   RAND_ADDR },
  { WRITE, RAND_ADDR, 0,     READ,   RAND_ADDR, WRITE, RAND_ADDR, 0,         READ, RAND_ADDR, READ,   RAND_ADDR },
}

function generateTestCase()
  -- First input is the initial value of memory
  local inputs = { math.random(-999, 999) }

  -- Pick two unique addresses for the templates
  local addr_1 = math.random(0, NUM_ADDR - 1)
  local addr_2 = addr_1
  while addr_2 == addr_1 do
    addr_2 = math.random(0, NUM_ADDR - 1)
  end

  -- Fill in the templates
  local template = TEMPLATES[math.random(1, #TEMPLATES)]
  for i = 1, #template do
    if template[i] == 0 then
      inputs[i + 1] = math.random(-999, 999)
    elseif template[i] == ADDR_1 then
      inputs[i + 1] = addr_1
    elseif template[i] == ADDR_2 then
      inputs[i + 1] = addr_2
    elseif template[i] == RAND_ADDR then
      inputs[i + 1] = math.random(0, NUM_ADDR - 1)
    else
      inputs[i + 1] = template[i]
    end
  end

  return inputs, execute(inputs)
end

function execute(inputs)
  local values = {}
  for i = 1, NUM_ADDR do
    values[i] = inputs[1]
  end

  -- Simulate the read-write commands
  local outputs = {}
  local i = 2
  while i <= #inputs do
    if inputs[i] == READ then
      i = i + 1
      outputs[#outputs + 1] = values[inputs[i] + 1]
    elseif inputs[i] == WRITE then
      i = i + 1
      values[inputs[i] + 1] = inputs[i + 1]
    end
    i = i + 1
  end

  return outputs
end
