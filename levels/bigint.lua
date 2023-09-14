function generateTestCase()
  local one = generateBigInt()
  local two = generateBigInt()

  -- Form the array [...one, -1, ...two]
  local inputs = { table.unpack(one) }
  inputs[#inputs + 1] = -1
  for i = 1, #two do
    inputs[#inputs + 1] = two[i]
  end

  local outputs = addBigInt(one, two)

  return inputs, outputs
end

function generateBigInt()
  local bigint = {}
  for i = 1, math.random(0, 4) do
    bigint[i] = math.random(0, 999)
  end

  -- Highest digit should not be a 0 unless it is the only digit
  if #bigint > 0 then
    bigint[#bigint + 1] = math.random(1, 999)
  else
    bigint[#bigint + 1] = math.random(0, 999)
  end

  return bigint
end

function addBigInt(a, b)
  -- Make a and b have the same digits
  for i = math.min(#a, #b), math.max(#a, #b) do
    if a[i] == nil then
      a[i] = 0
    end
    if b[i] == nil then
      b[i] = 0
    end
  end

  -- Compute the sum
  local carry = 0
  local sum = {}
  for i = 1, #a do
    local tmp = a[i] + b[i] + carry
    sum[i] = tmp % 1000
    carry = math.floor(tmp / 1000)
  end
  sum[#sum + 1] = carry

  -- Trim trailing zeros
  for i = #sum, 1, -1 do
    if sum[i] == 0 then
      sum[i] = nil
    end
  end

  return sum
end
