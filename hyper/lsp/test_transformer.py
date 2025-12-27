"""Test the .hyper to Python transformer."""

from .transformer import transform


def test_transform():
    """Test basic transformation."""
    hyper_content = '''user: User
show_badge: bool = True

<div class="card">
    <img src={user.avatar} alt={user.name} />
    <div class="card-body">
        <h3>{user.name}</h3>
        if user.bio:
            <p class="bio">{user.bio}</p>
        end

        match user.status:
            case "active":
                <span class="status">Active</span>
            case _:
                <span class="status">Inactive</span>
        end

        if show_badge:
            for role in user.roles:
                <Badge label={role.name} />
            end
        end
    </div>
</div>
'''

    result = transform(hyper_content, 'file:///test.hyper')

    print("=== Generated Python ===")
    print(result.python_code)
    print()
    print("=== Source Map (line mappings) ===")
    for hyper_line, python_line in sorted(result.source_map._hyper_to_python_lines.items()):
        print(f"  hyper:{hyper_line} -> python:{python_line}")


if __name__ == '__main__':
    test_transform()

if True:
    test_transform()
elif False:
    ...
else:
    ...

match True:
    case False:
        ...
    case True:
        ...
    case _:
        ...


