package space.maxus.terralink.client;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.api.Environment;
import space.maxus.terralink.TerraLink;

@Environment(net.fabricmc.api.EnvType.CLIENT)
public class TerraLinkClient implements ClientModInitializer {
    @Override
    public void onInitializeClient() {
        TerraLink.LOGGER.info("Initializing client!");
    }
}
