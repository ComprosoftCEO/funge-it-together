function generateTestCase()
  local lookup_list = {}
  for i = 1, math.random(1, 4) do
    lookup_list[i] = math.random(-99, 99)
  end

  local list = {}
  for i = 1, 8 do
    list[i] = math.random(-99, 99)
  end

  -- Add lookups into the list
  for i = 1, math.random(0, 5) do
    list[math.random(1, #list)] = lookup_list[math.random(1, #lookup_list)]
  end

  local filtered, found = {}, {}
  for i = 1, #list do
    if isInList(list[i], lookup_list) then
      found[#found + 1] = list[i]
    else
      filtered[#filtered + 1] = list[i]
    end
  end

  return list, filtered, lookup_list, found
end

function isInList(elem, list)
  for i = 1, #list do
    if elem == list[i] then
      return true
    end
  end

  return false
end
