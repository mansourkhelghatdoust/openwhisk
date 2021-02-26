/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.apache.openwhisk.core.database

import java.time.Instant
import akka.actor.ActorSystem
import akka.stream.ActorMaterializer
import org.apache.http.client.methods.HttpPost
import org.apache.http.entity.StringEntity
import org.apache.http.impl.client.HttpClientBuilder
import org.apache.openwhisk.common.{Logging, TransactionId}
import org.apache.openwhisk.core.entity._
import spray.json.JsObject

import scala.concurrent.Future
import scala.util.{Failure, Success}

class ArtifactActivationStore(actorSystem: ActorSystem, actorMaterializer: ActorMaterializer, logging: Logging)
    extends ActivationStore {

  implicit val executionContext = actorSystem.dispatcher

  private val artifactStore: ArtifactStore[WhiskActivation] =
    WhiskActivationStore.datastore()(actorSystem, logging, actorMaterializer)

  def store(activation: WhiskActivation, context: UserContext)(
    implicit transid: TransactionId,
    notifier: Option[CacheChangeNotification]): Future[DocInfo] = {

    logging.debug(this, s"recording activation '${activation.activationId}'")
    logging.error(this, s"ACTIVATED FUNCTION '${activation.name}', time ${activation.duration}")

    val nres = listActivationsMatchingName(activation.namespace, activation.name.toPath, 0, 200, context = context)

    nres onComplete {
      case Success(Right(activations)) =>

        val avg = expMovingAvg(activations.sortBy(a => a.start.getEpochSecond).map(a => a.duration.get));
        doLog(activation.name.asString, avg, activation.duration.getOrElse(0))
      case Success(Left(activations)) =>

        val avg = expMovingAvg(activations.sortBy(a => a.fields("start").toString().toLong).map(a => a.fields("duration").toString().toLong));
        doLog(activation.name.asString, avg, activation.duration.getOrElse(0))

      case Failure(exception) => logging.error(from=this, message="FAILED TO CALCULATE THE VALUES")
    }

    val res = WhiskActivation.put(artifactStore, activation)

    res onComplete {
      case Success(id) => logging.debug(this, s"recorded activation")
      case Failure(t) =>
        logging.error(
          this,
          s"failed to record activation ${activation.activationId} with error ${t.getLocalizedMessage}")
    }

    res

  }

  def doLog(action: String, estimated: Long, actual: Long): Unit = {
    logging.error(from=this, message=s"Estimated time is ${estimated}, actual was: ${actual}")
    val client = HttpClientBuilder.create().build();
    val post = new HttpPost("http://localhost:8000/logs");
    post.setHeader("Content-Type", "application/json");
    val body =s"""
         {\"action\": \"${action}\",
          \"estimated\": ${estimated},
          \"actual\": ${actual}
         }"""
    post.setEntity(new StringEntity(body))

    val resp = client.execute(post);
    logging.error(from=this, s"RESPONSE " + resp.getStatusLine.toString)
  }

  def expMovingAvg(data: List[Long]): Long = {
    if (data.isEmpty) {
      return 0
    }
    val alpha = 0.5;

    var avg = data.head.asInstanceOf[Double];

    for (i <- 1 until data.length) {
      avg = alpha * data(i) + (1.0 - alpha) * avg;
    }

    avg.asInstanceOf[Long]
  }

  def get(activationId: ActivationId, context: UserContext)(
    implicit transid: TransactionId): Future[WhiskActivation] = {
    WhiskActivation.get(artifactStore, DocId(activationId.asString))
  }

  /**
   * Here there is added overhead of retrieving the specified activation before deleting it, so this method should not
   * be used in production or performance related code.
   */
  def delete(activationId: ActivationId, context: UserContext)(
    implicit transid: TransactionId,
    notifier: Option[CacheChangeNotification]): Future[Boolean] = {
    WhiskActivation.get(artifactStore, DocId(activationId.asString)) flatMap { doc =>
      WhiskActivation.del(artifactStore, doc.docinfo)
    }
  }

  def countActivationsInNamespace(namespace: EntityPath,
                                  name: Option[EntityPath] = None,
                                  skip: Int,
                                  since: Option[Instant] = None,
                                  upto: Option[Instant] = None,
                                  context: UserContext)(implicit transid: TransactionId): Future[JsObject] = {
    WhiskActivation.countCollectionInNamespace(
      artifactStore,
      name.map(p => namespace.addPath(p)).getOrElse(namespace),
      skip,
      since,
      upto,
      StaleParameter.UpdateAfter,
      name.map(_ => WhiskActivation.filtersView).getOrElse(WhiskActivation.view))
  }

  def listActivationsMatchingName(
    namespace: EntityPath,
    name: EntityPath,
    skip: Int,
    limit: Int,
    includeDocs: Boolean = false,
    since: Option[Instant] = None,
    upto: Option[Instant] = None,
    context: UserContext)(implicit transid: TransactionId): Future[Either[List[JsObject], List[WhiskActivation]]] = {
    WhiskActivation.listActivationsMatchingName(
      artifactStore,
      namespace,
      name,
      skip,
      limit,
      includeDocs,
      since,
      upto,
      StaleParameter.UpdateAfter)
  }

  def listActivationsInNamespace(
    namespace: EntityPath,
    skip: Int,
    limit: Int,
    includeDocs: Boolean = false,
    since: Option[Instant] = None,
    upto: Option[Instant] = None,
    context: UserContext)(implicit transid: TransactionId): Future[Either[List[JsObject], List[WhiskActivation]]] = {
    WhiskActivation.listCollectionInNamespace(
      artifactStore,
      namespace,
      skip,
      limit,
      includeDocs,
      since,
      upto,
      StaleParameter.UpdateAfter)
  }

}

object ArtifactActivationStoreProvider extends ActivationStoreProvider {
  override def instance(actorSystem: ActorSystem, actorMaterializer: ActorMaterializer, logging: Logging) =
    new ArtifactActivationStore(actorSystem, actorMaterializer, logging)
}
