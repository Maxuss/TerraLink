package space.maxus.terralink;

import net.fabricmc.api.ModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import space.maxus.terralink.net.LinkerClient;

public class TerraLink implements ModInitializer {
    public static Logger LOGGER = LoggerFactory.getLogger("TerraLink");
    public static LinkerClient CLIENT;

    @Override
    public void onInitialize() {
        LOGGER.info("Initializing TerraLink...");
        CLIENT = new LinkerClient();
        LinkerClient.networkExecutor.execute(CLIENT::connect);
    }
}
