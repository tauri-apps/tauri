const fs = require('fs')
const path = require('path')
const schema = JSON.parse(
  fs.readFileSync('tooling/cli.rs/schema.json').toString()
)
const templatePath = path.join(__dirname, '../../docs/.templates/config.md')
const targetPath = path.join(__dirname, '../../docs/api/config.md')
const template = fs.readFileSync(templatePath, 'utf8')

function formatDescription(description) {
  return description
    ? description
        .replace(/`/g, '\\`')
        .replace(/\n/g, ' ')
        .replace(/  /g, ' ')
        .replace(/{/g, '\\{')
        .replace(/}/g, '\\}')
    : ''
}

function generatePropertiesEl(schema, anchorRoot, definition, tab) {
  const previousTabLevel = tab.replace('  ', '')
  const fields = [`anchorRoot="${anchorRoot}"`]

  if (definition.additionalProperties) {
    fields.push(`type="${definition.type}"`)
    fields.push(`description="${formatDescription(definition.description)}"`)
  }

  const rows = []
  for (const propertyName in definition.properties) {
    const property = definition.properties[propertyName]
    if ('type' in property) {
      let type
      if ('items' in property) {
        if (property.items.type) {
          type = `${property.items.type}[]`
        } else {
          const typeName = property.items.$ref.replace('#/definitions/', '')
          const propDefinition = schema.definitions[typeName]
          const propertyEl = generatePropertiesEl(
            schema,
            `${anchorRoot}.${propertyName}`,
            propDefinition,
            `${tab}  `
          )
          rows.push({
            property: propertyName,
            optional: 'default' in property || property.type.includes('null'),
            type: `${typeName}[]`,
            description: property.description,
            child: `<Array type="${typeName}">\n${tab}${propertyEl}\n${previousTabLevel}</Array>`
          })
          continue
        }
      } else if (Array.isArray(property.type)) {
        type = property.type.join(' | ')
      } else {
        type = property.type
      }
      rows.push({
        property: propertyName,
        optional: true,
        type,
        description: property.description,
        default: property.default
      })
    } else if ('anyOf' in property) {
      const subType = property.anyOf[0].$ref.replace('#/definitions/', '')
      const propDefinition = schema.definitions[subType]
      const propertyEl = generatePropertiesEl(
        schema,
        `${anchorRoot}.${propertyName}`,
        propDefinition,
        `${tab}  `
      )
      rows.push({
        property: propertyName,
        optional:
          property.anyOf.length > 1 && property.anyOf[1].type === 'null',
        type: subType,
        description: property.description,
        child: propertyEl
      })
    } else if ('allOf' in property) {
      const subType = property.allOf[0].$ref.replace('#/definitions/', '')
      const propDefinition = schema.definitions[subType]
      const propertyEl = propDefinition.properties
        ? generatePropertiesEl(
            schema,
            `${anchorRoot}.${propertyName}`,
            propDefinition,
            `${tab}  `
          )
        : undefined
      rows.push({
        property: propertyName,
        optional: 'default' in property,
        type: property.type || subType,
        description: property.description,
        child: propertyEl
      })
    }
  }

  if (rows.length > 0) {
    const serializedRows = rows
      .map((row) => {
        const fields = [
          `property: "${row.property}"`,
          `optional: ${row.optional}`,
          `type: "${row.type}"`,
          `description: \`${formatDescription(row.description)}\``
        ]
        if (row.child) {
          fields.push(`child: ${row.child}`)
        }
        return `{ ${fields.join(', ')} },`
      })
      .join(`\n${tab}`)
    fields.push(`rows={[\n${tab}${serializedRows}\n${previousTabLevel}]}`)
  } else {
    fields.push('rows={[]}')
  }

  return `<Properties ${fields.join(' ')}/>`
}

const output = []

for (const propertyName in schema.properties) {
  const property = schema.properties[propertyName]
  const definitionName = property.allOf[0].$ref.replace('#/definitions/', '')
  const definition = schema.definitions[definitionName]
  let contents = `## \`${propertyName}\`\n\n${generatePropertiesEl(
    schema,
    propertyName,
    definition,
    '  '
  )}`
  output.push(contents)
}

fs.writeFileSync(
  targetPath,
  template.replace('{properties}', output.join('\n\n'))
)
