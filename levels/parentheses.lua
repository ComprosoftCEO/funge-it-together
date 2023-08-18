local VALID_TEMPLATES = {} -- Generated below

function generateTestCase()
  if math.random(0, 1) == 1 then
    return VALID_TEMPLATES[math.random(1, #VALID_TEMPLATES)], { 1 }
  else
    return randomParentheses(), { 0 }
  end
end

function randomParentheses()
  local x = {}
  for i = 1, math.random(1, 15) do
    x[#x + 1] = ({ 1, -1 })[math.random(1, 2)]
  end

  return x
end

function generateParentheses(n)
  local function backtrack(arr, open, close)
    if #arr == 2 * n then
      coroutine.yield(arr)
      return
    end

    if open < n then
      table.insert(arr, 1)
      backtrack(arr, open + 1, close)
      table.remove(arr)
    end

    if close < open then
      table.insert(arr, -1)
      backtrack(arr, open, close + 1)
      table.remove(arr)
    end
  end

  return coroutine.wrap(function() backtrack({}, 0, 0) end)
end

-- Generate all valid matching parenthesis pairs
for len = 0, 6 do
  for valid in generateParentheses(len) do
    -- Construct [1, ...valid, 1]
    local array = { 1, table.unpack(valid) }
    array[#array + 1] = -1

    VALID_TEMPLATES[#VALID_TEMPLATES + 1] = array
  end
end
