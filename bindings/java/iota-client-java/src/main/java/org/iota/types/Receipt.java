package org.iota.types;

import com.google.gson.JsonObject;

public class Receipt extends AbstractObject {

    public Receipt(JsonObject jsonObject) {
        super(jsonObject);
    }

    public Receipt(String jsonObject) {
        super(jsonObject);
    }

}