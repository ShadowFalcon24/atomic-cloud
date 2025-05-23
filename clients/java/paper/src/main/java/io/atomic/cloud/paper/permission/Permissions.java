package io.atomic.cloud.paper.permission;

import io.papermc.paper.command.brigadier.CommandSourceStack;
import lombok.AllArgsConstructor;
import lombok.Getter;
import org.bukkit.permissions.Permissible;
import org.jetbrains.annotations.NotNull;

@AllArgsConstructor
@Getter
@SuppressWarnings("UnstableApiUsage")
public enum Permissions {
    CLOUD_COMMAND("atomic.cloud.command.cloud"),
    DISPOSE_COMMAND("atomic.cloud.command.dispose");

    private final String permission;

    public boolean check(@NotNull CommandSourceStack sourceStack) {
        return this.check(sourceStack.getSender());
    }

    public boolean check(@NotNull Permissible permissible) {
        return permissible.hasPermission(this.permission) || permissible.isOp(); // TODO: Remove isOp
    }
}
