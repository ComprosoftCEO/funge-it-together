local MESSAGES = {
  "aiwillrule",
  "nvrgiveyouup",
  "aiiscoming",
  "computercode",
  "obeyurmasters",
  "theendisnear",
  "datascience",
}

function generateTestCase()
  local key = math.random(1, 25)
  if math.random(2) == 1 then
    key = -1 * key
  end

  local message = MESSAGES[math.random(1, #MESSAGES)]
  local chars = getChars(message)

  local inputs = { key }
  local outputs = {}
  for i = 1, #chars do
    inputs[i + 1] = chars[i] + 1
    outputs[i] = ((chars[i] + key) % 26) + 1
  end

  return inputs, outputs
end

function getChars(input)
  local chars = {}
  for i = 1, #input do
    chars[i] = string.byte(input:sub(i, i)) - string.byte("a")
  end
  return chars
end
