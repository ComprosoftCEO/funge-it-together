local HIGHEST_FIB = 9

-- Cache the maximum allowed value for each k-Fibonacci
local MAX_K_FIBONACCI = {}
function fillLookupTable()
  for f = 2, HIGHEST_FIB do
    -- Inefficient but I don't care Lua is fast!
    for i = f, 999999 do
      if computeKFibonacci(f, i) > 999 then
        MAX_K_FIBONACCI[f] = i - 1
        break
      end
    end
  end
end

function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 14, 2 do
    inputs[i] = math.random(2, HIGHEST_FIB)
    inputs[i + 1] = math.random(0, MAX_K_FIBONACCI[inputs[i]])
    outputs[#outputs + 1] = computeKFibonacci(inputs[i], inputs[i + 1])
  end

  return inputs, outputs
end

-- Find f_k(n), where
--   f_k(0) = 0, k_k(1) = 1, ..., f_k(k-2) = 0
--   f_k(k-1) = 1
--   f_k(n) = f(n-1) + f(n-2) + ... + f_k(n-k)
function computeKFibonacci(k, n)
  local values = {}
  for i = 1, (k - 1) do
    values[i] = 0
  end
  values[k] = 1

  for _ = 1, n do
    local sum = 0
    for j = 1, #values - 1 do
      sum = sum + values[j]
      values[j] = values[j + 1]
    end
    values[#values] = values[#values] + sum
  end

  return values[1]
end

fillLookupTable()
